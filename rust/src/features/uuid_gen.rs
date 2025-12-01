use crate::state::{AppState, StringCharset};
use crate::ui::{Button as UiButton, Column as UiColumn, Text as UiText, TextInput as UiTextInput, maybe_push_back};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde_json::{json, Value};
use uuid::Uuid;

pub fn render_uuid_screen(state: &AppState) -> Value {
    let mut children = vec![
        serde_json::to_value(UiText::new("UUID & Random String").size(20.0)).unwrap(),
        serde_json::to_value(
            UiText::new("Generate UUID v4 or custom random strings.")
                .size(14.0),
        )
        .unwrap(),
        serde_json::to_value(UiButton::new("Generate UUID v4", "uuid_generate")).unwrap(),
    ];

    if let Some(u) = &state.uuid_generator.last_uuid {
        children.push(
            serde_json::to_value(
                UiText::new(u)
                    .size(14.0)
                    .content_description("uuid_value"),
            )
            .unwrap(),
        );
        children.push(
            serde_json::to_value(UiButton::new("Copy UUID", "copy_clipboard").copy_text(u)).unwrap(),
        );
    }

    children.push(serde_json::to_value(UiText::new("Random string").size(16.0)).unwrap());
    children.push(
        serde_json::to_value(
            UiTextInput::new("uuid_str_len")
                .hint("Length (e.g., 16)")
                .text(&state.uuid_generator.string_length.to_string())
                .single_line(true),
        )
        .unwrap(),
    );

    let charset_options = [
        (StringCharset::Alphanumeric, "Alphanumeric"),
        (StringCharset::Numeric, "Numeric"),
        (StringCharset::Alpha, "Alphabetic"),
        (StringCharset::Hex, "Hex"),
    ];
    for (charset, label) in charset_options {
        children.push(json!({
            "type": "Button",
            "text": label,
            "action": "random_string_charset",
            "content_description": if charset == state.uuid_generator.string_charset { Some("selected") } else { None::<&str> },
            "payload": { "charset": label }
        }));
    }

    children.push(serde_json::to_value(UiButton::new("Generate string", "random_string_generate")).unwrap());

    if let Some(s) = &state.uuid_generator.last_string {
        children.push(
            serde_json::to_value(
                UiText::new(s)
                    .size(14.0)
                    .content_description("uuid_random_string"),
            )
            .unwrap(),
        );
        children.push(
            serde_json::to_value(UiButton::new("Copy string", "copy_clipboard").copy_text(s))
                .unwrap(),
        );
    }

    maybe_push_back(&mut children, state);
    serde_json::to_value(UiColumn::new(children).padding(20)).unwrap()
}

pub fn handle_uuid_action(state: &mut AppState, action: &str, bindings: &std::collections::HashMap<String, String>) {
    match action {
        "uuid_generate" => {
            state.uuid_generator.last_uuid = Some(generate_uuid());
        }
        "random_string_charset" => {
            if let Some(label) = bindings.get("charset") {
                if let Some(parsed) = parse_charset_label(label) {
                    state.uuid_generator.string_charset = parsed;
                }
            }
        }
        "random_string_generate" => {
            let len = bindings
                .get("uuid_str_len")
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(state.uuid_generator.string_length)
                .max(1)
                .min(512);
            state.uuid_generator.string_length = len;
            let s = generate_string(len as usize, state.uuid_generator.string_charset);
            state.uuid_generator.last_string = Some(s);
        }
        _ => {}
    }
}

pub fn generate_uuid() -> String {
    Uuid::new_v4().to_string()
}

pub fn generate_string(len: usize, charset: StringCharset) -> String {
    let rng = thread_rng();
    match charset {
        StringCharset::Alphanumeric => rng
            .sample_iter(&Alphanumeric)
            .take(len)
            .map(char::from)
            .collect(),
        StringCharset::Numeric => rng
            .sample_iter(rand::distributions::Uniform::new_inclusive(b'0', b'9'))
            .take(len)
            .map(|b| b as char)
            .collect(),
        StringCharset::Alpha => rng
            .sample_iter(rand::distributions::Uniform::new_inclusive(b'a', b'z'))
            .take(len)
            .map(|b| b as char)
            .collect(),
        StringCharset::Hex => rng
            .sample_iter(rand::distributions::Uniform::new_inclusive(b'0', b'f'))
            .take(len)
            .map(|b| {
                let c = b as char;
                if c >= 'g' {
                    (b'0' + (b % 16)) as char
                } else {
                    c
                }
            })
            .collect(),
    }
}

fn parse_charset_label(label: &str) -> Option<StringCharset> {
    match label.to_lowercase().as_str() {
        "alphanumeric" => Some(StringCharset::Alphanumeric),
        "numeric" => Some(StringCharset::Numeric),
        "alphabetic" | "alpha" => Some(StringCharset::Alpha),
        "hex" => Some(StringCharset::Hex),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uuid_v4_is_valid() {
        let u = generate_uuid();
        assert_eq!(u.len(), 36);
        assert!(u.chars().filter(|c| *c == '-').count() == 4);
    }

    #[test]
    fn random_string_respects_length_and_charset() {
        let s = generate_string(10, StringCharset::Numeric);
        assert_eq!(s.len(), 10);
        assert!(s.chars().all(|c| c.is_ascii_digit()));
    }
}
