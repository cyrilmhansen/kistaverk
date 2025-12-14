use crate::state::AppState;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::ffi::{CStr, CString};
use std::ptr;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::io::Read;
use std::os::unix::io::FromRawFd;
use libc::{self, c_int, c_void, c_char};
use mir_sys::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CScriptingState {
    pub source: String,
    pub args: String,
    pub output: String,
    pub error: Option<String>,
    pub is_running: bool,
}

impl CScriptingState {
    pub const fn new() -> Self {
        Self {
            source: String::new(),
            args: String::new(),
            output: String::new(),
            error: None,
            is_running: false,
        }
    }

    pub fn execute(&mut self, timeout_ms: u64) {
        self.output.clear();
        self.error = None;
        self.is_running = true;

        let source = self.source.clone();
        let args = self.args.clone();
        let (tx, rx) = mpsc::channel();

        // Spawn a thread to run the C execution
        thread::spawn(move || {
            let result = execute_c_code_in_thread(source, args);
            let _ = tx.send(result);
        });

        // Wait for result with timeout
        match rx.recv_timeout(Duration::from_millis(timeout_ms)) {
            Ok(Ok(output)) => {
                self.output = output;
            }
            Ok(Err(err)) => {
                self.error = Some(err);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                self.error = Some(format!("Execution timed out after {}ms", timeout_ms));
                // Note: The worker thread is detached and might still be running.
                // In a robust implementation, we would need a way to signal it to stop.
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                self.error = Some("Worker thread disconnected unexpectedly".to_string());
            }
        }
        self.is_running = false;
    }
}

struct StringReader {
    data: Vec<u8>,
    cursor: usize,
}

unsafe extern "C" fn getc_func(data: *mut c_void) -> c_int {
    let reader = &mut *(data as *mut StringReader);
    if reader.cursor < reader.data.len() {
        let byte = reader.data[reader.cursor];
        reader.cursor += 1;
        byte as c_int
    } else {
        -1 // EOF
    }
}

fn execute_c_code_in_thread(source_code: String, args_str: String) -> Result<String, String> {
    unsafe {
        // 1. Setup output capturing (pipe)
        let mut pipe_fds: [c_int; 2] = [0; 2];
        if libc::pipe(pipe_fds.as_mut_ptr()) < 0 {
            return Err("Failed to create pipe".to_string());
        }
        let (read_fd, write_fd) = (pipe_fds[0], pipe_fds[1]);

        // Save original stdout/stderr
        let original_stdout = libc::dup(libc::STDOUT_FILENO);
        let original_stderr = libc::dup(libc::STDERR_FILENO);

        // Redirect stdout/stderr to pipe
        libc::dup2(write_fd, libc::STDOUT_FILENO);
        libc::dup2(write_fd, libc::STDERR_FILENO);
        
        // We can close the write end in this process now, but we need it open for the duration of the C code.
        // Actually, we keep it open, and only close it after C code is done so we can read EOF.

        // 2. Initialize MIR
        #[cfg(target_os = "android")]
        let ctx = _MIR_init(ptr::null_mut(), ptr::null_mut());
        #[cfg(not(target_os = "android"))]
        let ctx = _MIR_init(ptr::null_mut(), ptr::null_mut()); // Simplify for now

        if ctx.is_null() {
            // Restore stdout/stderr before returning
            libc::dup2(original_stdout, libc::STDOUT_FILENO);
            libc::dup2(original_stderr, libc::STDERR_FILENO);
            libc::close(original_stdout);
            libc::close(original_stderr);
            libc::close(write_fd);
            libc::close(read_fd);
            return Err("Failed to initialize MIR context".to_string());
        }

        MIR_gen_init(ctx);
        MIR_gen_set_optimize_level(ctx, 1);
        c2mir_init(ctx);

        // 3. Register standard functions
        // For security, strictly we should limit what we load, but for general C scripting we load libc basics.
        // On Android, symbols might be available or need explicit loading.
        MIR_load_external(ctx, CString::new("printf").unwrap().as_ptr(), libc::printf as *mut c_void);
        MIR_load_external(ctx, CString::new("puts").unwrap().as_ptr(), libc::puts as *mut c_void);
        MIR_load_external(ctx, CString::new("malloc").unwrap().as_ptr(), libc::malloc as *mut c_void);
        MIR_load_external(ctx, CString::new("free").unwrap().as_ptr(), libc::free as *mut c_void);
        MIR_load_external(ctx, CString::new("strtol").unwrap().as_ptr(), libc::strtol as *mut c_void);
        MIR_load_external(ctx, CString::new("atoi").unwrap().as_ptr(), libc::atoi as *mut c_void);
        // Add more as needed

        // 4. Compile C source
        let mut reader = StringReader {
            data: source_code.bytes().collect(),
            cursor: 0,
        };
        let mut options: c2mir_options = std::mem::zeroed();
        
        let compile_result = c2mir_compile(
            ctx,
            &mut options,
            Some(getc_func),
            &mut reader as *mut _ as *mut c_void,
            b"script.c\0".as_ptr() as *const _,
            ptr::null_mut(),
        );

        let execution_result = if compile_result == 1 {
             // 5. Link and Run
            let module_list = MIR_get_module_list(ctx);
            let module = (*module_list).tail;
            if module.is_null() {
                 Err("Compilation success but no module found".to_string())
            } else {
                MIR_load_module(ctx, module);
                MIR_link(ctx, Some(MIR_set_gen_interface), None);

                let main_name = CString::new("main").unwrap();
                let mut func_item = (*module).items.head;
                let mut found_func = ptr::null_mut();

                while !func_item.is_null() {
                    if (*func_item).item_type == MIR_item_type_t_MIR_func_item {
                        let name_ptr = MIR_item_name(ctx, func_item);
                        let name = CStr::from_ptr(name_ptr);
                        if name == main_name.as_c_str() {
                            found_func = func_item;
                            break;
                        }
                    }
                    func_item = (*func_item).item_link.next;
                }

                if found_func.is_null() {
                    Err("Function 'main' not found".to_string())
                } else {
                    let fun_ptr = MIR_gen(ctx, found_func);
                    if fun_ptr.is_null() {
                         Err("MIR generation failed".to_string())
                    } else {
                        // Prepare argv
                        // argv[0] = "script"
                        let prog_name = CString::new("script").unwrap();
                        let mut argv_cstrings: Vec<CString> = vec![prog_name];
                        
                        // Parse user args (simple split by whitespace)
                        for arg in args_str.split_whitespace() {
                            if let Ok(c_arg) = CString::new(arg) {
                                argv_cstrings.push(c_arg);
                            }
                        }
                        
                        // Create array of pointers
                        let mut argv_ptrs: Vec<*mut c_char> = argv_cstrings.iter()
                            .map(|cs| cs.as_ptr() as *mut c_char)
                            .collect();
                        argv_ptrs.push(ptr::null_mut()); // NULL terminator

                        let argc = argv_cstrings.len() as c_int;
                        let argv = argv_ptrs.as_mut_ptr();

                        // Call main(argc, argv)
                        let rust_func: extern "C" fn(c_int, *mut *mut c_char) -> c_int = std::mem::transmute(fun_ptr);
                        let _ret = rust_func(argc, argv);
                        Ok(())
                    }
                }
            }
        } else {
            Err("Compilation failed".to_string())
        };

        // 6. Cleanup MIR
        c2mir_finish(ctx);
        MIR_gen_finish(ctx);
        MIR_finish(ctx);

        // 7. Restore stdout/stderr
        // Flush stdout/stderr to ensure everything is in the pipe
        libc::fflush(ptr::null_mut()); 

        libc::dup2(original_stdout, libc::STDOUT_FILENO);
        libc::dup2(original_stderr, libc::STDERR_FILENO);
        libc::close(original_stdout);
        libc::close(original_stderr);
        
        // Close write end of pipe to signal EOF to reader
        libc::close(write_fd);

        // 8. Read captured output
        let mut output = String::new();
        let mut file = std::fs::File::from_raw_fd(read_fd);
        let _ = file.read_to_string(&mut output);
        // file closes read_fd on drop

        match execution_result {
            Ok(_) => Ok(output),
            Err(e) => Err(format!("Error: {}\nOutput so far:\n{}", e, output)),
        }
    }
}

pub fn render_c_scripting_screen(state: &AppState) -> serde_json::Value {
    let cs = &state.c_scripting;

    let mut components = Vec::new();

    components.push(json!({
        "type": "Text",
        "text": "C Scripting Lab",
        "size": 24.0,
        "bold": true,
        "margin_bottom": 16.0
    }));

    components.push(json!({
        "type": "TextInput",
        "bind_key": "c_scripting.source",
        "text": cs.source,
        "hint": "int main(int argc, char **argv) { ... }",
        "single_line": false,
        "max_lines": 20,
        "margin_bottom": 12.0
    }));

    components.push(json!({
        "type": "Text",
        "text": "Program Arguments (space separated):",
        "size": 14.0,
        "margin_bottom": 6.0
    }));

    components.push(json!({
        "type": "TextInput",
        "bind_key": "c_scripting.args",
        "text": cs.args,
        "hint": "e.g. 2025 12 25",
        "single_line": true,
        "max_lines": 1,
        "margin_bottom": 12.0
    }));

    components.push(json!({
        "type": "Row",
        "children": [
            {
                "type": "Button",
                "text": "Run (10s limit)",
                "action": "c_scripting_execute",
                "margin_right": 8.0,
                "disabled": cs.is_running
            },
            {
                "type": "Button",
                "text": "Load Example",
                "action": "c_scripting_load_example",
                "disabled": cs.is_running
            }
        ]
    }));
    
    components.push(json!({
        "type": "Button",
        "text": "Clear Output",
        "action": "c_scripting_clear",
        "margin_top": 8.0,
        "margin_bottom": 16.0
    }));

    components.push(json!({
        "type": "Text",
        "text": "Output:",
        "size": 18.0,
        "bold": true,
        "margin_bottom": 8.0
    }));

    let output_text = if cs.is_running {
        "Running...".to_string()
    } else if let Some(error) = &cs.error {
        format!("Error: {}", error)
    } else if cs.output.is_empty() {
        "Ready.".to_string()
    } else {
        cs.output.clone()
    };

    components.push(json!({
        "type": "CodeView",
        "text": output_text,
        "wrap": true,
        "theme": "light",
        "line_numbers": false,
        "margin_bottom": 16.0
    }));

    json!({
        "type": "Column",
        "children": components
    })
}

pub fn handle_c_scripting_actions(
    state: &mut AppState,
    action: crate::router::Action,
) -> Option<serde_json::Value> {
    use crate::router::Action::*;

    match action {
        CScriptingScreen => {
            state.push_screen(crate::state::Screen::CScripting);
            Some(render_c_scripting_screen(state))
        }
        CScriptingExecute { source, args } => {
            state.c_scripting.source = source;
            if let Some(a) = args {
                state.c_scripting.args = a;
            }
            state.c_scripting.execute(10_000); // 10s default timeout
            Some(render_c_scripting_screen(state))
        }
        CScriptingClear => {
            state.c_scripting.output.clear();
            state.c_scripting.error = None;
            Some(render_c_scripting_screen(state))
        }
        CScriptingLoadExample => {
            state.c_scripting.args = "2025 12 25".to_string();
            state.c_scripting.source = r#"
#include <stdio.h>
#include <stdlib.h>

const char *days[] = {"Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"};

int wday(int year, int month, int day) {
    int adjustment, mm, yy;
    adjustment = (14 - month) / 12;
    mm = month + 12 * adjustment - 2;
    yy = year - adjustment;
    return (day + (13 * mm - 1) / 5 + yy + yy / 4 - yy / 100 + yy / 400) % 7;
}

int main(int argc, char **argv) {
    if (argc < 4) {
        printf("Usage: %s <year> <month> <day>\n", argv[0]);
        return 1;
    }

    char *end;
    long y = strtol(argv[1], &end, 10);
    if (*end != '\0') { printf("Invalid year: %s\n", argv[1]); return 1; }

    long m = strtol(argv[2], &end, 10);
    if (*end != '\0') { printf("Invalid month: %s\n", argv[2]); return 1; }

    long d = strtol(argv[3], &end, 10);
    if (*end != '\0') { printf("Invalid day: %s\n", argv[3]); return 1; }

    int wd = wday((int)y, (int)m, (int)d);
    printf("%04ld-%02ld-%02ld is a %s\n", y, m, d, days[wd]);

    return 0;
}
"#.trim().to_string();
            state.c_scripting.output.clear();
            Some(render_c_scripting_screen(state))
        }
        _ => None,
    }
}
