use crate::state::AppState;
use rhai::Engine;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptingState {
    pub script: String,
    pub output: String,
    pub error: Option<String>,
}

impl ScriptingState {
    pub const fn new() -> Self {
        Self {
            script: String::new(),
            output: String::new(),
            error: None,
        }
    }

    pub fn execute_script(&mut self) {
        let engine = Engine::new();
        
        // Clear previous output and error
        self.output.clear();
        self.error = None;
        
        // Execute the script and capture output
        match engine.eval::<String>(&self.script) {
            Ok(result) => {
                self.output = result;
            }
            Err(e) => {
                self.error = Some(format!("Script execution error: {}", e));
            }
        }
    }

    pub fn clear_output(&mut self) {
        self.output.clear();
        self.error = None;
    }

    pub fn clear_script(&mut self) {
        self.script.clear();
        self.clear_output();
    }
}

pub fn render_scripting_screen(state: &AppState) -> serde_json::Value {
    let scripting_state = &state.scripting;
    
    let mut components = Vec::new();
    
    // Title
    components.push(json!({
        "type": "Text",
        "text": "Scripting Lab",
        "size": 24.0,
        "bold": true,
        "margin_bottom": 16.0
    }));
    
    // Script editor
    components.push(json!({
        "type": "TextArea",
        "bind_key": "scripting.script",
        "text": scripting_state.script,
        "hint": "Enter your Rhai script here...",
        "min_lines": 10,
        "max_lines": 20,
        "margin_bottom": 12.0
    }));
    
    // Action buttons
    components.push(json!({
        "type": "Row",
        "children": [
            {
                "type": "Button",
                "text": "Execute",
                "action_id": "scripting.execute",
                "flex": 1,
                "margin_right": 8.0
            },
            {
                "type": "Button",
                "text": "Clear Output",
                "action_id": "scripting.clear_output",
                "flex": 1,
                "margin_right": 8.0
            },
            {
                "type": "Button",
                "text": "Clear Script",
                "action_id": "scripting.clear_script",
                "flex": 1
            }
        ]
    }));
    
    // Output section
    components.push(json!({
        "type": "Text",
        "text": "Output:",
        "size": 18.0,
        "bold": true,
        "margin_top": 16.0,
        "margin_bottom": 8.0
    }));
    
    // Output display
    let output_text = if let Some(error) = &scripting_state.error {
        format!("Error: {}", error)
    } else if scripting_state.output.is_empty() {
        "No output yet. Execute a script to see results.".to_string()
    } else {
        scripting_state.output.clone()
    };
    
    components.push(json!({
        "type": "TextArea",
        "text": output_text,
        "read_only": true,
        "min_lines": 5,
        "max_lines": 10,
        "background_color": "#f5f5f5",
        "margin_bottom": 16.0
    }));
    
    // Example scripts section
    components.push(json!({
        "type": "Text",
        "text": "Examples:",
        "size": 18.0,
        "bold": true,
        "margin_top": 16.0,
        "margin_bottom": 8.0
    }));
    
    components.push(json!({
        "type": "Column",
        "children": [
            {
                "type": "Button",
                "text": "Load: Hello World",
                "action_id": "scripting.load_example.hello",
                "margin_bottom": 4.0
            },
            {
                "type": "Button",
                "text": "Load: Math Operations",
                "action_id": "scripting.load_example.math",
                "margin_bottom": 4.0
            },
            {
                "type": "Button",
                "text": "Load: String Manipulation",
                "action_id": "scripting.load_example.string",
                "margin_bottom": 4.0
            },
            {
                "type": "Button",
                "text": "Load: Random + Loop",
                "action_id": "scripting.load_example.random_loop",
                "margin_bottom": 4.0
            },
            {
                "type": "Button",
                "text": "Load: Map & Array",
                "action_id": "scripting.load_example.collections",
                "margin_bottom": 4.0
            },
            {
                "type": "Button",
                "text": "Load: Function",
                "action_id": "scripting.load_example.function",
                "margin_bottom": 4.0
            }
        ]
    }));
    
    json!({
        "type": "Column",
        "children": components
    })
}

pub fn handle_scripting_actions(state: &mut AppState, action: crate::router::Action) -> Option<serde_json::Value> {
    use crate::router::Action::*;
    
    match action {
        ScriptingScreen => {
            state.push_screen(crate::state::Screen::Scripting);
            Some(render_scripting_screen(state))
        }
        ScriptingExecute => {
            state.scripting.execute_script();
            Some(render_scripting_screen(state))
        }
        ScriptingClearOutput => {
            state.scripting.clear_output();
            Some(render_scripting_screen(state))
        }
        ScriptingClearScript => {
            state.scripting.clear_script();
            Some(render_scripting_screen(state))
        }
        ScriptingLoadExample { example_type } => {
            load_example_script(state, &example_type);
            Some(render_scripting_screen(state))
        }
        _ => None,
    }
}

fn load_example_script(state: &mut AppState, example_type: &str) {
    let script = match example_type {
        "hello" => {
            r#""
// Hello World Example
"Hello, World! This is a simple Rhai script."
"#.to_string()
        }
        "math" => {
            r#""
// Math Operations Example
let x = 42;
let y = 10;
let sum = x + y;
let product = x * y;
let quotient = x / y;

"Math Results:\nSum: " + sum + "\nProduct: " + product + "\nQuotient: " + quotient
"#.to_string()
        }
        "string" => {
            r#""
// String Manipulation Example
let name = "Rhai";
let greeting = "Hello, " + name + "! Welcome to scripting.";
let upper = greeting.to_uppercase();
let lower = greeting.to_lowercase();
let length = greeting.len();

"Original: " + greeting + "\nUpper: " + upper + "\nLower: " + lower + "\nLength: " + length
"#.to_string()
        }
        "random_loop" => {
            r#""
// Loop & aggregate example
let mut squares = [];
let mut total = 0;
for n in 1..6 {
    let s = n * n;
    squares.push(s);
    total += s;
}
"Squares: " + squares.join(", ") + "\nTotal: " + total + "\nAverage: " + (total / squares.len())
"#.to_string()
        }
        "collections" => {
            r#""
// Map & Array Example
let user = #{ name: "Ada", role: "engineer" };
let tags = ["rust", "kotlin", "json"];
let mut tags_up = [];
for t in tags {
    tags_up.push(t.to_uppercase());
}
"User " + user["name"] + " (" + user["role"] + ")\nTags: " + tags.join(", ") + "\nTags upper: " + tags_up.join(", ")
"#.to_string()
        }
        "function" => {
            r#""
// Function Example
fn greet(name) {
    if name.len() == 0 { return "Hi, mystery user!"; }
    "Hi, " + name + "! The time is " + now()
}

fn now() { 
    // Rhai sandbox: provide a static timestamp string
    "123456"
}

greet("Kistaverk user")
"#.to_string()
        }
        _ => "// Unknown example type".to_string(),
    };
    
    state.scripting.script = script;
    state.scripting.clear_output();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;

    #[test]
    fn test_scripting_state_initialization() {
        let state = ScriptingState::new();
        assert_eq!(state.script, "");
        assert_eq!(state.output, "");
        assert_eq!(state.error, None);
    }

    #[test]
    fn test_scripting_execution() {
        let mut state = ScriptingState::new();
        state.script = r#"
let x = 42;
let y = 10;
let sum = x + y;
"Sum: " + sum
"#.to_string();

        state.execute_script();
        
        assert_eq!(state.output, "Sum: 52");
        assert_eq!(state.error, None);
    }

    #[test]
    fn test_scripting_error_handling() {
        let mut state = ScriptingState::new();
        state.script = "invalid syntax here!".to_string();

        state.execute_script();
        
        assert_eq!(state.output, "");
        assert!(state.error.is_some());
        assert!(state.error.unwrap().contains("Script execution error"));
    }

    #[test]
    fn test_clear_functions() {
        let mut state = ScriptingState::new();
        state.script = "some script".to_string();
        state.output = "some output".to_string();
        state.error = Some("some error".to_string());

        state.clear_output();
        assert_eq!(state.script, "some script");
        assert_eq!(state.output, "");
        assert_eq!(state.error, None);

        state.clear_script();
        assert_eq!(state.script, "");
        assert_eq!(state.output, "");
        assert_eq!(state.error, None);
    }

    #[test]
    fn test_example_scripts() {
        let mut app_state = AppState::new();
        
        // Test hello world example
        load_example_script(&mut app_state, "hello");
        assert!(app_state.scripting.script.contains("Hello, World"));
        assert_eq!(app_state.scripting.output, "");
        assert_eq!(app_state.scripting.error, None);
        
        // Test math example
        load_example_script(&mut app_state, "math");
        assert!(app_state.scripting.script.contains("Math Operations"));
        assert_eq!(app_state.scripting.output, "");
        assert_eq!(app_state.scripting.error, None);
        
        // Test string example
        load_example_script(&mut app_state, "string");
        assert!(app_state.scripting.script.contains("String Manipulation"));
        assert_eq!(app_state.scripting.output, "");
        assert_eq!(app_state.scripting.error, None);
        
        // Test unknown example
        load_example_script(&mut app_state, "unknown");
        assert_eq!(app_state.scripting.script, "// Unknown example type");

        // New examples
        load_example_script(&mut app_state, "random_loop");
        assert!(app_state.scripting.script.contains("Loop & aggregate"));
        load_example_script(&mut app_state, "collections");
        assert!(app_state.scripting.script.contains("Map & Array"));
        load_example_script(&mut app_state, "function");
        assert!(app_state.scripting.script.contains("Function Example"));
    }
}
