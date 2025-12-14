use crate::state::AppState;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::ffi::{CStr, CString};
use std::ptr;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

static MIR_GLOBAL_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[cfg(target_os = "android")]
fn logcat(msg: &str) {
    unsafe {
        let tag = b"kistaverk-mir\0";
        let c_msg = CString::new(msg).unwrap_or_else(|_| CString::new("<log msg had NUL>").unwrap());
        android_log_sys::__android_log_print(
            android_log_sys::LogPriority::INFO as _,
            tag.as_ptr() as *const _,
            b"%s\0".as_ptr() as *const _,
            c_msg.as_ptr(),
        );
    }
}

#[cfg(not(target_os = "android"))]
fn logcat(_msg: &str) {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirScriptingState {
    pub source: String,
    pub entry: String,
    pub output: String,
    pub error: Option<String>,
}

impl MirScriptingState {
    pub const fn new() -> Self {
        Self {
            source: String::new(),
            entry: String::new(),
            output: String::new(),
            error: None,
        }
    }

    pub fn execute_jit(&mut self) -> Option<u128> {
        self.execute_impl(MirExecMode::Jit)
    }

    pub fn execute_interp(&mut self) -> Option<u128> {
        self.execute_impl(MirExecMode::Interp)
    }

    fn execute_impl(&mut self, mode: MirExecMode) -> Option<u128> {
        self.output.clear();
        self.error = None;

        let _mir_guard = MIR_GLOBAL_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .ok();

        let entry = self.entry.trim().to_string();
        let entry = if entry.is_empty() { "main".to_string() } else { entry };

        logcat("MIR execute: start");
        logcat(&format!("MIR execute: entry={}", entry));
        logcat(&format!("MIR execute: mode={}", mode.as_str()));

        let mut normalized_source = self.source.replace("\r\n", "\n").replace('\r', "");
        // MIR scanner expects statements to be separated by NL or ';'. Ensure the last line ends
        // with a newline so `endmodule`/`endfunc` isn't treated as having trailing junk at EOF.
        if !normalized_source.ends_with('\n') {
            normalized_source.push('\n');
        }
        let cr_count = self.source.as_bytes().iter().filter(|&&b| b == b'\r').count();
        logcat(&format!(
            "MIR execute: source_bytes={} cr_count={}",
            self.source.as_bytes().len(),
            cr_count
        ));

        let source = match CString::new(normalized_source) {
            Ok(v) => v,
            Err(_) => {
                self.error = Some("MIR source contains a NUL byte".to_string());
                return None;
            }
        };

        let entry_c = match CString::new(entry.as_str()) {
            Ok(v) => v,
            Err(_) => {
                self.error = Some("Entry function name contains a NUL byte".to_string());
                return None;
            }
        };

        unsafe {
            let started = Instant::now();
            #[cfg(unix)]
            let mut code_alloc = mir_sys::code_alloc::unix_mmap();
            #[cfg(unix)]
            let ctx = mir_sys::_MIR_init(ptr::null_mut(), &mut code_alloc);
            #[cfg(not(unix))]
            let ctx = mir_sys::_MIR_init(ptr::null_mut(), ptr::null_mut());

            if ctx.is_null() {
                self.error = Some("Failed to initialize MIR context".to_string());
                return None;
            }

            logcat(&format!("MIR: ctx={:p}", ctx));
            logcat("MIR: MIR_gen_init");
            mir_sys::MIR_gen_init(ctx);
            logcat("MIR: MIR_gen_set_optimize_level(0)");
            mir_sys::MIR_gen_set_optimize_level(ctx, 0);

            logcat("MIR: MIR_scan_string");
            mir_sys::MIR_scan_string(ctx, source.as_ptr());
            logcat("MIR: MIR_scan_string done");

            logcat("MIR: MIR_get_module_list");
            let module_list_ptr = mir_sys::MIR_get_module_list(ctx);
            if module_list_ptr.is_null() {
                self.error = Some("Failed to read MIR module list".to_string());
                logcat("MIR: module_list_ptr is null");
                mir_sys::MIR_gen_finish(ctx);
                mir_sys::MIR_finish(ctx);
                return None;
            }

            let module = (*module_list_ptr).tail;
            if module.is_null() {
                self.error = Some("Failed to parse MIR module".to_string());
                logcat("MIR: module tail is null");
                mir_sys::MIR_gen_finish(ctx);
                mir_sys::MIR_finish(ctx);
                return None;
            }

            logcat(&format!("MIR: module={:p}", module));
            logcat("MIR: MIR_load_module");
            mir_sys::MIR_load_module(ctx, module);
            logcat("MIR: MIR_load_module done");

            let mut item = (*module).items.head;
            let mut found: mir_sys::MIR_item_t = ptr::null_mut();
            while !item.is_null() {
                if (*item).item_type == mir_sys::MIR_item_type_t_MIR_func_item {
                    let name_ptr = mir_sys::MIR_item_name(ctx, item);
                    if !name_ptr.is_null() {
                        let name = CStr::from_ptr(name_ptr);
                        if name == entry_c.as_c_str() {
                            found = item;
                            break;
                        }
                    }
                }
                item = (*item).item_link.next;
            }

            if found.is_null() {
                self.error = Some(format!("Function '{}' not found in module", entry));
                logcat("MIR: entry function not found");
                mir_sys::MIR_gen_finish(ctx);
                mir_sys::MIR_finish(ctx);
                return None;
            }

            logcat(&format!("MIR: found_func={:p}", found));
            let result = match mode {
                MirExecMode::Jit => {
                    logcat("MIR: MIR_link (JIT)");
                    mir_sys::MIR_link(ctx, Some(mir_sys::MIR_set_gen_interface), None);
                    logcat("MIR: MIR_link done");

                    logcat("MIR: MIR_gen");
                    let fun_ptr = mir_sys::MIR_gen(ctx, found);
                    if fun_ptr.is_null() {
                        self.error = Some("MIR code generation failed".to_string());
                        logcat("MIR: MIR_gen returned null");
                        mir_sys::MIR_gen_finish(ctx);
                        mir_sys::MIR_finish(ctx);
                        return None;
                    }

                    logcat(&format!("MIR: fun_ptr={:p}", fun_ptr));
                    let rust_func: extern "C" fn() -> i64 = std::mem::transmute(fun_ptr);
                    logcat("MIR: calling generated function");
                    rust_func()
                }
                MirExecMode::Interp => {
                    logcat("MIR: MIR_set_interp_interface");
                    mir_sys::MIR_set_interp_interface(ctx, found);
                    let mut out = mir_sys::MIR_val_t { i: 0 };
                    logcat("MIR: MIR_interp_arr");
                    mir_sys::MIR_interp_arr(ctx, found, &mut out as *mut _, 0, ptr::null_mut());
                    out.i
                }
            };

            logcat(&format!("MIR: function returned {}", result));
            self.output = format!("Result: {}", result);

            logcat("MIR: MIR_gen_finish");
            mir_sys::MIR_gen_finish(ctx);
            logcat("MIR: MIR_finish");
            mir_sys::MIR_finish(ctx);
            logcat("MIR execute: done");
            Some(started.elapsed().as_millis())
        }
    }


    pub fn clear_output(&mut self) {
        self.output.clear();
        self.error = None;
    }

    pub fn clear_source(&mut self) {
        self.source.clear();
        self.clear_output();
    }
}

#[derive(Clone, Copy, Debug)]
enum MirExecMode {
    Jit,
    Interp,
}

impl MirExecMode {
    fn as_str(self) -> &'static str {
        match self {
            MirExecMode::Jit => "jit",
            MirExecMode::Interp => "interp",
        }
    }
}

pub fn render_mir_scripting_screen(state: &AppState) -> serde_json::Value {
    let ms = &state.mir_scripting;
    let entry = ms.entry.clone().if_empty_then("main");

    let mut components = Vec::new();

    components.push(json!({
        "type": "Text",
        "text": "MIR Scripting Lab",
        "size": 24.0,
        "bold": true,
        "margin_bottom": 16.0
    }));

    components.push(json!({
        "type": "Text",
        "text": "Entry function (called as: extern \"C\" fn() -> i64):",
        "size": 14.0,
        "margin_bottom": 6.0
    }));

    components.push(json!({
        "type": "TextInput",
        "bind_key": "mir_scripting.entry",
        "text": entry,
        "hint": "main",
        "single_line": true,
        "max_lines": 1,
        "margin_bottom": 12.0
    }));

    components.push(json!({
        "type": "TextInput",
        "bind_key": "mir_scripting.source",
        "text": ms.source,
        "hint": "Enter your MIR module here...",
        "single_line": false,
        "max_lines": 24,
        "margin_bottom": 12.0
    }));

    components.push(json!({
        "type": "Column",
        "children": [
            {
                "type": "Button",
                "text": "Run (Interpreter)",
                "action": "mir_scripting_execute_interp",
                "margin_bottom": 8.0
            },
            {
                "type": "Button",
                "text": "Run (JIT)",
                "action": "mir_scripting_execute_jit",
                "margin_bottom": 8.0
            },
            {
                "type": "Button",
                "text": "Clear Output",
                "action": "mir_scripting_clear_output",
                "margin_bottom": 8.0
            },
            {
                "type": "Button",
                "text": "Clear Source",
                "action": "mir_scripting_clear_source",
                "margin_bottom": 8.0
            },
            {
                "type": "Button",
                "text": "Load Example",
                "action": "mir_scripting_load_example"
            }
        ]
    }));

    components.push(json!({
        "type": "Text",
        "text": "Output:",
        "size": 18.0,
        "bold": true,
        "margin_top": 16.0,
        "margin_bottom": 8.0
    }));

    let output_text = if let Some(error) = &ms.error {
        format!("Error: {}", error)
    } else if ms.output.is_empty() {
        "No output yet. Execute to see results.".to_string()
    } else {
        ms.output.clone()
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

pub fn handle_mir_scripting_actions(
    state: &mut AppState,
    action: crate::router::Action,
) -> Option<serde_json::Value> {
    use crate::router::Action::*;

    match action {
        MirScriptingScreen => {
            state.push_screen(crate::state::Screen::MirScripting);
            Some(render_mir_scripting_screen(state))
        }
        MirScriptingExecuteJit { source, entry } => {
            state.mir_scripting.source = source;
            state.mir_scripting.entry = entry;
            let runtime_ms = state.mir_scripting.execute_jit();
            if let Some(ms) = runtime_ms {
                state.toast = Some(format!("MIR JIT runtime: {} ms", ms));
            }
            Some(render_mir_scripting_screen(state))
        }
        MirScriptingExecuteInterp { source, entry } => {
            state.mir_scripting.source = source;
            state.mir_scripting.entry = entry;
            let runtime_ms = state.mir_scripting.execute_interp();
            if let Some(ms) = runtime_ms {
                state.toast = Some(format!("MIR interpreter runtime: {} ms", ms));
            }
            Some(render_mir_scripting_screen(state))
        }
        MirScriptingClearOutput => {
            state.mir_scripting.clear_output();
            Some(render_mir_scripting_screen(state))
        }
        MirScriptingClearSource => {
            state.mir_scripting.clear_source();
            Some(render_mir_scripting_screen(state))
        }
        MirScriptingLoadExample => {
            state.mir_scripting.entry = "ex100".to_string();
            state.mir_scripting.source = r#"
m_sieve:  module
          export sieve
sieve:    func i32, i32:N
          local i64:iter, i64:count, i64:i, i64:k, i64:prime, i64:temp, i64:flags
          alloca flags, 819000
          mov iter, 0
loop:     bge fin, iter, N
          mov count, 0;  mov i, 0
loop2:    bge fin2, i, 819000
          mov u8:(flags, i), 1;  add i, i, 1
          jmp loop2
fin2:     mov i, 0
loop3:    bge fin3, i, 819000
          beq cont3, u8:(flags,i), 0
          add temp, i, i;  add prime, temp, 3;  add k, i, prime
loop4:    bge fin4, k, 819000
          mov u8:(flags, k), 0;  add k, k, prime
          jmp loop4
fin4:     add count, count, 1
cont3:    add i, i, 1
          jmp loop3
fin3:     add iter, iter, 1
          jmp loop
fin:      ret count
          endfunc
          endmodule

m_ex100:  module
          export ex100
          import sieve
p_sieve:  proto i32, i32:iter
ex100:    func i64
          local i64:r
          call p_sieve, sieve, r, 100
          ret r
          endfunc
          endmodule
"#
            .trim()
            .to_string();
            state.mir_scripting.clear_output();
            Some(render_mir_scripting_screen(state))
        }
        _ => None,
    }
}

trait IfEmptyThen {
    fn if_empty_then(self, default: &str) -> Self;
}

impl IfEmptyThen for String {
    fn if_empty_then(self, default: &str) -> Self {
        if self.trim().is_empty() {
            default.to_string()
        } else {
            self
        }
    }
}
