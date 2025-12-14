use crate::state::AppState;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::ffi::{CStr, CString};
use std::ptr;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use std::io::Read;
use std::os::unix::io::FromRawFd;
use libc::{self, c_int, c_void, c_char};
use mir_sys::*;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CScriptingState {
    pub source: String,
    pub args: String,
    pub output: String,
    pub error: Option<String>,
    pub is_running: bool,
    pub use_jit: bool,
    pub benchmark: bool,
    pub run_in_thread: bool,
    pub compilation_time_us: Option<u128>,
    pub avg_execution_time_us: Option<u128>,
    pub execution_count: u32,
}

impl CScriptingState {
    pub const fn new() -> Self {
        Self {
            source: String::new(),
            args: String::new(),
            output: String::new(),
            error: None,
            is_running: false,
            use_jit: false, // Default to interpreter for stability
            benchmark: false,
            run_in_thread: true,
            compilation_time_us: None,
            avg_execution_time_us: None,
            execution_count: 0,
        }
    }

    pub fn execute(&mut self, timeout_ms: u64) {
        self.output.clear();
        self.error = None;
        self.is_running = true;
        self.compilation_time_us = None;
        self.avg_execution_time_us = None;
        self.execution_count = 0;

        let source = self.source.clone();
        let args = self.args.clone();
        let use_jit = self.use_jit;
        let benchmark = self.benchmark;
        let (tx, rx) = mpsc::channel();

        // Spawn a thread to run the C execution
        thread::spawn(move || {
            let result = execute_c_code(source, args, use_jit, benchmark);
            let _ = tx.send(result);
        });

        // Wait for result with timeout
        // Increase timeout for benchmarks
        let effective_timeout = if benchmark { 30_000 } else { timeout_ms };
        
        match rx.recv_timeout(Duration::from_millis(effective_timeout)) {
            Ok(result) => {
                match result {
                    Ok(exec_res) => {
                        self.output = exec_res.output;
                        if benchmark {
                            self.compilation_time_us = Some(exec_res.compilation_time_us);
                            self.avg_execution_time_us = Some(exec_res.avg_execution_time_us);
                            self.execution_count = exec_res.runs;
                        }
                    }
                    Err(err) => {
                        self.error = Some(err);
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                self.error = Some("Error. 10 seconds timeout over".to_string());
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

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub output: String,
    pub compilation_time_us: u128,
    pub avg_execution_time_us: u128,
    pub runs: u32,
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

/// Generates a new UUID v4 and returns a pointer to a C-string.
/// The caller is responsible for freeing the string using `free_rust_string()`.
unsafe extern "C" fn uuid_v4_gen() -> *mut c_char {
    let id = Uuid::new_v4();
    match CString::new(id.to_string()) {
        Ok(s) => s.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// Frees a C-string that was allocated by Rust (e.g. via `uuid_v4_gen`).
unsafe extern "C" fn free_rust_string(s: *mut c_char) {
    if !s.is_null() {
        let _ = CString::from_raw(s);
    }
}

pub fn execute_c_code(source_code: String, args_str: String, use_jit: bool, benchmark: bool) -> Result<ExecutionResult, String> {
    unsafe {
        // 1. Setup output capturing (pipe)
        let mut pipe_fds: [c_int; 2] = [0; 2];
        if libc::pipe(pipe_fds.as_mut_ptr()) < 0 {
            return Err("Failed to create pipe".to_string());
        }
        let (read_fd, write_fd) = (pipe_fds[0], pipe_fds[1]);

        // Start a thread to read from the pipe immediately to prevent buffer filling and deadlock
        let (out_tx, out_rx) = mpsc::channel();
        let mut reader_file = std::fs::File::from_raw_fd(read_fd);
        thread::spawn(move || {
            let mut buf = String::new();
            let _ = reader_file.read_to_string(&mut buf);
            let _ = out_tx.send(buf);
        });

        // Save original stdout/stderr
        let original_stdout = libc::dup(libc::STDOUT_FILENO);
        let original_stderr = libc::dup(libc::STDERR_FILENO);

        // Redirect stdout/stderr to pipe
        libc::dup2(write_fd, libc::STDOUT_FILENO);
        libc::dup2(write_fd, libc::STDERR_FILENO);
        
        // Measure Compilation Start
        let compile_start = Instant::now();

        // 2. Initialize MIR
        let ctx = _MIR_init(ptr::null_mut(), ptr::null_mut());

        if ctx.is_null() {
            // Restore stdout/stderr before returning
            libc::dup2(original_stdout, libc::STDOUT_FILENO);
            libc::dup2(original_stderr, libc::STDERR_FILENO);
            libc::close(original_stdout);
            libc::close(original_stderr);
            libc::close(write_fd);
            return Err("Failed to initialize MIR context".to_string());
        }

        MIR_gen_init(ctx);
        MIR_gen_set_optimize_level(ctx, 1);
        c2mir_init(ctx);

        // 3. Register standard functions
        MIR_load_external(ctx, CString::new("printf").unwrap().as_ptr(), libc::printf as *mut c_void);
        MIR_load_external(ctx, CString::new("puts").unwrap().as_ptr(), libc::puts as *mut c_void);
        MIR_load_external(ctx, CString::new("malloc").unwrap().as_ptr(), libc::malloc as *mut c_void);
        MIR_load_external(ctx, CString::new("free").unwrap().as_ptr(), libc::free as *mut c_void);
        MIR_load_external(ctx, CString::new("strtol").unwrap().as_ptr(), libc::strtol as *mut c_void);
        MIR_load_external(ctx, CString::new("atoi").unwrap().as_ptr(), libc::atoi as *mut c_void);
        MIR_load_external(ctx, CString::new("fflush").unwrap().as_ptr(), libc::fflush as *mut c_void);
        
        // Register custom functions
        MIR_load_external(ctx, CString::new("uuid_v4_gen").unwrap().as_ptr(), uuid_v4_gen as *mut c_void);
        MIR_load_external(ctx, CString::new("free_rust_string").unwrap().as_ptr(), free_rust_string as *mut c_void);

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

        let mut compilation_time_us = 0;
        let mut avg_execution_time_us = 0;
        let mut runs = 0;

        let execution_result = if compile_result == 1 {
             // 5. Link and Run
            let module_list = MIR_get_module_list(ctx);
            let module = (*module_list).tail;
            if module.is_null() {
                 Err("Compilation success but no module found".to_string())
            } else {
                MIR_load_module(ctx, module);
                
                if use_jit {
                    MIR_link(ctx, Some(MIR_set_gen_interface), None);
                } else {
                    MIR_link(ctx, Some(MIR_set_interp_interface), None);
                }

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
                    let fun_ptr = if use_jit {
                        MIR_gen(ctx, found_func)
                    } else {
                        (*found_func).addr
                    };

                    if fun_ptr.is_null() {
                         Err("Failed to get function address".to_string())
                    } else {
                        // Measure Compilation End
                        compilation_time_us = compile_start.elapsed().as_micros();

                        // Prepare argv
                        let prog_name = CString::new("script").unwrap();
                        let mut argv_cstrings: Vec<CString> = vec![prog_name];
                        
                        for arg in args_str.split_whitespace() {
                            if let Ok(c_arg) = CString::new(arg) {
                                argv_cstrings.push(c_arg);
                            }
                        }
                        
                        let mut argv_ptrs: Vec<*mut c_char> = argv_cstrings.iter()
                            .map(|cs| cs.as_ptr() as *mut c_char)
                            .collect();
                        argv_ptrs.push(ptr::null_mut()); 

                        let argc = argv_cstrings.len() as c_int;
                        let argv = argv_ptrs.as_mut_ptr();

                        let rust_func: extern "C" fn(c_int, *mut *mut c_char) -> c_int = std::mem::transmute(fun_ptr);
                        
                        // Execution Loop
                        let limit_runs = if benchmark { 100 } else { 1 };
                        let limit_time = Duration::from_secs(10);
                        let mut total_exec_time = Duration::ZERO;
                        let mut last_ret = 0;

                        for _ in 0..limit_runs {
                            let exec_start = Instant::now();
                            last_ret = rust_func(argc, argv);
                            total_exec_time += exec_start.elapsed();
                            runs += 1;

                            if benchmark && total_exec_time >= limit_time {
                                break;
                            }
                        }

                        if runs > 0 {
                            avg_execution_time_us = total_exec_time.as_micros() / runs as u128;
                        }

                        // We only return result of last run? Or just Ok(())
                        Ok(last_ret)
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
        libc::fflush(ptr::null_mut()); 

        libc::dup2(original_stdout, libc::STDOUT_FILENO);
        libc::dup2(original_stderr, libc::STDERR_FILENO);
        libc::close(original_stdout);
        libc::close(original_stderr);
        libc::close(write_fd);

        // 8. Read captured output
        let output = out_rx.recv().unwrap_or_default();

        match execution_result {
            Ok(_) => Ok(ExecutionResult {
                output,
                compilation_time_us,
                avg_execution_time_us,
                runs,
            }),
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
                "type": "Checkbox",
                "text": "Use JIT (Faster)",
                "bind_key": "c_scripting_use_jit",
                "checked": cs.use_jit,
                "action": "c_scripting_toggle_jit",
                "margin_right": 16.0
            },
            {
                "type": "Checkbox",
                "text": "Benchmark",
                "bind_key": "c_scripting_benchmark",
                "checked": cs.benchmark,
                "action": "c_scripting_toggle_benchmark"
            },
            {
                "type": "Checkbox",
                "text": "Run in background",
                "bind_key": "c_scripting_run_in_thread",
                "checked": cs.run_in_thread,
                "action": "c_scripting_toggle_thread"
            }
        ],
        "margin_bottom": 12.0
    }));

    components.push(json!({
        "type": "Row",
        "children": [
            {
                "type": "Button",
                "text": if cs.benchmark { "Run Benchmark" } else { "Run (10s limit)" },
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

    if cs.benchmark && cs.compilation_time_us.is_some() {
        components.push(json!({
            "type": "Card",
            "title": "Benchmark Results",
            "children": [
                {
                    "type": "Text",
                    "text": format!("Compilation: {:.3} ms", cs.compilation_time_us.unwrap() as f64 / 1000.0),
                    "size": 14.0,
                    "margin_bottom": 4.0
                },
                {
                    "type": "Text",
                    "text": format!("Avg Execution: {:.3} ms ({} runs)", cs.avg_execution_time_us.unwrap() as f64 / 1000.0, cs.execution_count),
                    "size": 14.0,
                    "margin_bottom": 4.0
                }
            ],
            "margin_bottom": 16.0
        }));
    }

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
            state.c_scripting.compilation_time_us = None;
            state.c_scripting.avg_execution_time_us = None;
            Some(render_c_scripting_screen(state))
        }
        CScriptingLoadExample => {
            state.c_scripting.args = "".to_string();
            state.c_scripting.source = r#"
// Prototype for the Rust-exported function
char* uuid_v4_gen();
void free_rust_string(char* s);

int main(int argc, char **argv) {
    printf("Invoking 'uuid' crate from C...\n");
    
    char* id = uuid_v4_gen();
    if (id) {
        printf("Generated UUID: %s\n", id);
        // Important: Free the memory allocated by Rust
        free_rust_string(id);
    } else {
        printf("Failed to generate UUID.\n");
    }

    return 0;
}
"#.trim().to_string();
            state.c_scripting.output.clear();
            Some(render_c_scripting_screen(state))
        }
        CScriptingToggleJit { enabled } => {
            state.c_scripting.use_jit = enabled;
            Some(render_c_scripting_screen(state))
        }
        CScriptingToggleBenchmark { enabled } => {
            state.c_scripting.benchmark = enabled;
            Some(render_c_scripting_screen(state))
        }
        CScriptingToggleThread { enabled } => {
            state.c_scripting.run_in_thread = enabled;
            Some(render_c_scripting_screen(state))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_script_execution() {
        let source = r#"
            int main(int argc, char **argv) {
                printf("Hello from C!\n");
                return 42;
            }
        "#.to_string();
        
        let args = "".to_string();
        let (tx, rx) = mpsc::channel();
        
        thread::spawn(move || {
            let result = execute_c_code(source, args, false, false);
            let _ = tx.send(result);
        });

        match rx.recv_timeout(Duration::from_secs(5)) {
            Ok(result) => {
                match result {
                    Ok(res) => {
                        if !res.output.contains("Hello from C!") {
                            panic!("Expected 'Hello from C!' in output, but got: '{}'", res.output);
                        }
                    },
                    Err(e) => panic!("C execution failed: {}", e),
                }
            }
            Err(_) => panic!("Test timed out after 5 seconds"),
        }
    }

    #[test]
    fn test_c_script_jit_execution() {
        let source = r#"
            int printf(const char *format, ...);
            int fflush(void *stream);
            
            int main(int argc, char **argv) {
                printf("Hello JIT!\n");
                fflush(0);
                return 0;
            }
        "#.to_string();
        
        let (tx, rx) = mpsc::channel();
        
        thread::spawn(move || {
            let result = execute_c_code(source, "".to_string(), true, false);
            let _ = tx.send(result);
        });

        match rx.recv_timeout(Duration::from_secs(5)) {
            Ok(result) => {
                match result {
                    Ok(res) => {
                        if !res.output.contains("Hello JIT!") {
                            panic!("Expected 'Hello JIT!' in output, but got: '{}'", res.output);
                        }
                    },
                    Err(e) => panic!("C JIT execution failed: {}", e),
                }
            }
            Err(_) => panic!("Test timed out after 5 seconds"),
        }
    }
}

