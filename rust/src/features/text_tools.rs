use crate::state::{AppState, Screen};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Handle text tool actions by updating state based on the provided bindings.
pub fn handle_text_action(state: &mut AppState, action: &str, bindings: &HashMap<String, String>) {
    if let Some(input) = bindings.get("text_input") {
        state.text_input = Some(input.clone());
    }

    if let Some(flag) = parse_bool(bindings.get("aggressive_trim")) {
        state.text_aggressive_trim = flag;
    }

    let input = state.text_input.clone().unwrap_or_default();
    state.current_screen = Screen::TextTools;

    match action {
        "text_tools_upper" => {
            state.text_output = Some(input.to_uppercase());
            state.text_operation = Some("UPPERCASE".into());
        }
        "text_tools_lower" => {
            state.text_output = Some(input.to_lowercase());
            state.text_operation = Some("lowercase".into());
        }
        "text_tools_title" => {
            let title = input
                .split_whitespace()
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                        None => String::new(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
            state.text_output = Some(title);
            state.text_operation = Some("Title Case".into());
        }
        "text_tools_word_count" => {
            let count = input
                .split_whitespace()
                .filter(|part| !part.is_empty())
                .count();
            state.text_output = Some(format!("Word count: {}", count));
            state.text_operation = Some("Word count".into());
        }
        "text_tools_char_count" => {
            let count = input.chars().count();
            state.text_output = Some(format!("Character count: {}", count));
            state.text_operation = Some("Character count".into());
        }
        "text_tools_trim" => {
            let trimmed = if state.text_aggressive_trim {
                input.split_whitespace().collect::<Vec<_>>().join(" ")
            } else {
                input.trim().to_string()
            };
            state.text_output = Some(trimmed);
            state.text_operation = Some(if state.text_aggressive_trim {
                "Trim spacing (collapse)".into()
            } else {
                "Trim edges".into()
            });
        }
        "text_tools_wrap" => {
            let wrapped = wrap_text(&input, 72);
            state.text_output = Some(wrapped);
            state.text_operation = Some("Wrap to 72 cols".into());
        }
        "text_tools_clear" => {
            state.text_input = Some(String::new());
            state.text_output = None;
            state.text_operation = Some("Cleared".into());
        }
        "text_tools_refresh" => {
            // No-op: used to capture bindings (e.g., checkbox toggles) and re-render.
            state.text_operation = state.text_operation.take();
        }
        _ => {}
    }
}

fn parse_bool(value: Option<&String>) -> Option<bool> {
    value.and_then(|v| {
        let lower = v.to_ascii_lowercase();
        match lower.as_str() {
            "true" | "1" | "yes" | "on" => Some(true),
            "false" | "0" | "no" | "off" => Some(false),
            _ => None,
        }
    })
}

fn wrap_text(input: &str, width: usize) -> String {
    if width == 0 {
        return input.to_string();
    }

    let mut lines = Vec::new();
    let mut current = String::new();

    for word in input.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
            continue;
        }

        if current.len() + 1 + word.len() <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current);
            current = word.to_string();
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }

    lines.join("\n")
}

pub fn render_text_tools_screen(state: &AppState) -> Value {
    let input = state.text_input.clone().unwrap_or_default();
    let mut children = vec![
        json!({
            "type": "Text",
            "text": "Text tools",
            "size": 20.0
        }),
        json!({
            "type": "Text",
            "text": "Enter text, then apply a transform or count.",
            "size": 14.0
        }),
        json!({
            "type": "TextInput",
            "bind_key": "text_input",
            "text": input,
            "hint": "Paste or type text",
            "content_description": "Input text for text tools"
        }),
        json!({
            "type": "Column",
            "padding": 8,
            "children": [
                { "type": "Text", "text": "Transforms", "size": 14.0 },
                { "type": "Button", "text": "UPPERCASE", "action": "text_tools_upper" },
                { "type": "Button", "text": "lowercase", "action": "text_tools_lower" },
                { "type": "Button", "text": "Title Case", "action": "text_tools_title" }
            ]
        }),
        json!({
            "type": "Column",
            "padding": 8,
            "children": [
                { "type": "Text", "text": "Counts & cleanup", "size": 14.0 },
                {
                    "type": "Checkbox",
                    "text": "Aggressive trim (collapse whitespace)",
                    "bind_key": "aggressive_trim",
                    "checked": state.text_aggressive_trim,
                    "action": "text_tools_refresh"
                },
                { "type": "Button", "text": "Word count", "action": "text_tools_word_count" },
                { "type": "Button", "text": "Character count", "action": "text_tools_char_count" },
                { "type": "Button", "text": "Trim spacing", "action": "text_tools_trim" },
                { "type": "Button", "text": "Wrap to 72 cols", "action": "text_tools_wrap" },
                { "type": "Button", "text": "Clear", "action": "text_tools_clear" }
            ]
        }),
    ];

    if let Some(op) = &state.text_operation {
        children.push(json!({
            "type": "Text",
            "text": format!("Last action: {}", op),
            "size": 14.0
        }));
    }

    if let Some(result) = &state.text_output {
        children.push(json!({
            "type": "Column",
            "padding": 8,
            "children": [
                { "type": "Text", "text": "Result", "size": 14.0 },
                { "type": "Text", "text": result, "size": 16.0 }
            ]
        }));
    }

    children.push(json!({
        "type": "Button",
        "text": "Back",
        "action": "reset"
    }));

    json!({
        "type": "Column",
        "padding": 24,
        "children": children
    })
}
