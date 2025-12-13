use crate::state::AppState;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::ffi::{CStr, CString};
use std::ptr;

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

    pub fn execute(&mut self) {
        self.output.clear();
        self.error = None;

        let entry = self.entry.trim();
        let entry = if entry.is_empty() { "main" } else { entry };

        let source = match CString::new(self.source.clone()) {
            Ok(v) => v,
            Err(_) => {
                self.error = Some("MIR source contains a NUL byte".to_string());
                return;
            }
        };

        let entry_c = match CString::new(entry) {
            Ok(v) => v,
            Err(_) => {
                self.error = Some("Entry function name contains a NUL byte".to_string());
                return;
            }
        };

        unsafe {
            #[cfg(unix)]
            let mut code_alloc = mir_sys::code_alloc::unix_mmap();
            #[cfg(unix)]
            let ctx = mir_sys::_MIR_init(ptr::null_mut(), &mut code_alloc);
            #[cfg(not(unix))]
            let ctx = mir_sys::_MIR_init(ptr::null_mut(), ptr::null_mut());

            if ctx.is_null() {
                self.error = Some("Failed to initialize MIR context".to_string());
                return;
            }

            mir_sys::MIR_gen_init(ctx);
            mir_sys::MIR_gen_set_optimize_level(ctx, 0);

            mir_sys::MIR_scan_string(ctx, source.as_ptr());

            let module_list_ptr = mir_sys::MIR_get_module_list(ctx);
            if module_list_ptr.is_null() {
                self.error = Some("Failed to read MIR module list".to_string());
                mir_sys::MIR_gen_finish(ctx);
                mir_sys::MIR_finish(ctx);
                return;
            }

            let module = (*module_list_ptr).tail;
            if module.is_null() {
                self.error = Some("Failed to parse MIR module".to_string());
                mir_sys::MIR_gen_finish(ctx);
                mir_sys::MIR_finish(ctx);
                return;
            }

            mir_sys::MIR_load_module(ctx, module);

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
                mir_sys::MIR_gen_finish(ctx);
                mir_sys::MIR_finish(ctx);
                return;
            }

            mir_sys::MIR_link(ctx, Some(mir_sys::MIR_set_gen_interface), None);

            let fun_ptr = mir_sys::MIR_gen(ctx, found);
            if fun_ptr.is_null() {
                self.error = Some("MIR code generation failed".to_string());
                mir_sys::MIR_gen_finish(ctx);
                mir_sys::MIR_finish(ctx);
                return;
            }

            let rust_func: extern "C" fn() -> i64 = std::mem::transmute(fun_ptr);
            let result = rust_func();
            self.output = format!("Result: {}", result);

            mir_sys::MIR_gen_finish(ctx);
            mir_sys::MIR_finish(ctx);
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
                "text": "Execute",
                "action": "mir_scripting_execute",
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
        MirScriptingExecute { source, entry } => {
            state.mir_scripting.source = source;
            state.mir_scripting.entry = entry;
            state.mir_scripting.execute();
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
            state.mir_scripting.entry = "main".to_string();
            state.mir_scripting.source = r#"
m_calc:   module
          export main
main:     func i64
          local i64:r
          mov r, 150
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
