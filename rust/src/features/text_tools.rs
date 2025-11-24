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
        "text_tools_base64_encode" => {
            state.text_output = Some(encode_base64(input.as_bytes()));
            state.text_operation = Some("Base64 encode".into());
        }
        "text_tools_base64_decode" => match decode_base64(input.as_bytes()) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(s) => {
                    state.text_output = Some(s);
                    state.text_operation = Some("Base64 decode".into());
                }
                Err(_) => {
                    state.text_output = Some("<non-UTF8 data>".into());
                    state.text_operation = Some("Base64 decode (binary)".into());
                }
            },
            Err(e) => {
                state.text_output = Some(format!("Decode error: {e}"));
                state.text_operation = Some("Base64 decode failed".into());
            }
        },
        "text_tools_url_encode" => {
            state.text_output = Some(url_encode(&input));
            state.text_operation = Some("URL encode".into());
        }
        "text_tools_url_decode" => match url_decode(&input) {
            Ok(s) => {
                state.text_output = Some(s);
                state.text_operation = Some("URL decode".into());
            }
            Err(e) => {
                state.text_output = Some(format!("Decode error: {e}"));
                state.text_operation = Some("URL decode failed".into());
            }
        },
        "text_tools_hex_encode" => {
            state.text_output = Some(hex_encode(input.as_bytes()));
            state.text_operation = Some("Hex encode".into());
        }
        "text_tools_hex_decode" => match hex_decode(&input) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(s) => {
                    state.text_output = Some(s);
                    state.text_operation = Some("Hex decode".into());
                }
                Err(_) => {
                    state.text_output = Some("<non-UTF8 data>".into());
                    state.text_operation = Some("Hex decode (binary)".into());
                }
            },
            Err(e) => {
                state.text_output = Some(format!("Decode error: {e}"));
                state.text_operation = Some("Hex decode failed".into());
            }
        },
        "text_tools_copy_to_input" => {
            if let Some(result) = state.text_output.clone() {
                state.text_input = Some(result);
                state.text_operation = Some("Result copied to input".into());
            }
        }
        "text_tools_share_result" => {
            state.text_operation = Some("Share result tapped".into());
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

fn encode_base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    let mut chunks = bytes.chunks(3);
    while let Some(chunk) = chunks.next() {
        let b0 = chunk.get(0).copied().unwrap_or(0);
        let b1 = chunk.get(1).copied().unwrap_or(0);
        let b2 = chunk.get(2).copied().unwrap_or(0);
        let n = ((b0 as u32) << 16) | ((b1 as u32) << 8) | b2 as u32;
        out.push(TABLE[((n >> 18) & 0x3f) as usize] as char);
        out.push(TABLE[((n >> 12) & 0x3f) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[((n >> 6) & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(n & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

fn decode_base64(input: &[u8]) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 4];
    let mut idx = 0;
    for &b in input {
        if b == b'=' || b == b'\r' || b == b'\n' || b == b' ' {
            continue;
        }
        let val = match b {
            b'A'..=b'Z' => b - b'A',
            b'a'..=b'z' => b - b'a' + 26,
            b'0'..=b'9' => b - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return Err("invalid_base64_char".into()),
        };
        chunk[idx] = val;
        idx += 1;
        if idx == 4 {
            let n = ((chunk[0] as u32) << 18)
                | ((chunk[1] as u32) << 12)
                | ((chunk[2] as u32) << 6)
                | (chunk[3] as u32);
            buf.push(((n >> 16) & 0xff) as u8);
            buf.push(((n >> 8) & 0xff) as u8);
            buf.push((n & 0xff) as u8);
            idx = 0;
        }
    }
    if idx == 2 {
        let n = ((chunk[0] as u32) << 18) | ((chunk[1] as u32) << 12);
        buf.push(((n >> 16) & 0xff) as u8);
    } else if idx == 3 {
        let n = ((chunk[0] as u32) << 18) | ((chunk[1] as u32) << 12) | ((chunk[2] as u32) << 6);
        buf.push(((n >> 16) & 0xff) as u8);
        buf.push(((n >> 8) & 0xff) as u8);
    }
    Ok(buf)
}

fn url_encode(input: &str) -> String {
    fn is_unreserved(byte: u8) -> bool {
        (byte.is_ascii_alphanumeric()) || matches!(byte, b'-' | b'_' | b'.' | b'~')
    }
    let mut out = String::new();
    for b in input.as_bytes() {
        if is_unreserved(*b) {
            out.push(*b as char);
        } else {
            out.push_str(&format!("%{:02X}", b));
        }
    }
    out
}

fn url_decode(input: &str) -> Result<String, String> {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).map_err(|_| "invalid_utf8")?;
                let val = u8::from_str_radix(hex, 16).map_err(|_| "invalid_hex")?;
                out.push(val);
                i += 3;
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            other => {
                out.push(other);
                i += 1;
            }
        }
    }
    String::from_utf8(out).map_err(|_| "invalid_utf8_output".into())
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

fn hex_decode(input: &str) -> Result<Vec<u8>, String> {
    let trimmed = input.trim();
    if trimmed.len() % 2 != 0 {
        return Err("invalid_hex_length".into());
    }
    let mut out = Vec::with_capacity(trimmed.len() / 2);
    let chars: Vec<char> = trimmed.chars().collect();
    for i in (0..chars.len()).step_by(2) {
        let hi = chars[i].to_digit(16).ok_or_else(|| "invalid_hex_digit")?;
        let lo = chars[i + 1]
            .to_digit(16)
            .ok_or_else(|| "invalid_hex_digit")?;
        out.push(((hi << 4) | lo) as u8);
    }
    Ok(out)
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
                { "type": "Button", "text": "Base64 encode", "action": "text_tools_base64_encode" },
                { "type": "Button", "text": "Base64 decode", "action": "text_tools_base64_decode" },
                { "type": "Button", "text": "URL encode", "action": "text_tools_url_encode" },
                { "type": "Button", "text": "URL decode", "action": "text_tools_url_decode" },
                { "type": "Button", "text": "Hex encode", "action": "text_tools_hex_encode" },
                { "type": "Button", "text": "Hex decode", "action": "text_tools_hex_decode" },
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
        children.push(json!({
            "type": "Column",
            "padding": 8,
            "children": [
                { "type": "Text", "text": "Result actions", "size": 14.0 },
                { "type": "Button", "text": "Copy to input", "action": "text_tools_copy_to_input" },
                { "type": "Button", "text": "Share result", "action": "text_tools_share_result" }
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
