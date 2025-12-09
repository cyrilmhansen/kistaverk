use crate::state::AppState;
use crate::ui::{maybe_push_back, Button as UiButton, CodeView as UiCodeView, Column as UiColumn, Text as UiText, TextInput as UiTextInput};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtState {
    pub input_token: String,
    pub decoded_header: Option<String>,
    pub decoded_payload: Option<String>,
    pub error: Option<String>,
}

impl JwtState {
    pub const fn new() -> Self {
        Self {
            input_token: String::new(),
            decoded_header: None,
            decoded_payload: None,
            error: None,
        }
    }
}

pub fn decode_jwt(token: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() < 2 {
        return Err("jwt_invalid_parts".into());
    }
    let header_raw = URL_SAFE_NO_PAD
        .decode(parts[0])
        .map_err(|e| format!("jwt_header_b64_error:{e}"))?;
    let payload_raw = URL_SAFE_NO_PAD
        .decode(parts[1])
        .map_err(|e| format!("jwt_payload_b64_error:{e}"))?;

    let header_json: Value =
        serde_json::from_slice(&header_raw).map_err(|e| format!("jwt_header_json_error:{e}"))?;
    let payload_json: Value = serde_json::from_slice(&payload_raw)
        .map_err(|e| format!("jwt_payload_json_error:{e}"))?;

    let header_pretty =
        serde_json::to_string_pretty(&header_json).map_err(|e| format!("jwt_header_fmt:{e}"))?;
    let payload_pretty =
        serde_json::to_string_pretty(&payload_json).map_err(|e| format!("jwt_payload_fmt:{e}"))?;

    Ok((header_pretty, payload_pretty))
}

pub fn render_jwt_screen(state: &AppState) -> serde_json::Value {
    let mut children = vec![
        serde_json::to_value(UiText::new("JWT Decoder").size(20.0)).unwrap(),
        serde_json::to_value(
            UiText::new("Paste a JWT to inspect its header and payload.").size(14.0),
        )
        .unwrap(),
        serde_json::to_value(
            UiTextInput::new("jwt_input")
                .text(&state.jwt.input_token)
                .hint("eyJhbGciOi...")
                .max_lines(4),
        )
        .unwrap(),
        serde_json::to_value(UiButton::new("Decode", "jwt_decode")).unwrap(),
        serde_json::to_value(UiButton::new("Clear", "jwt_clear")).unwrap(),
        serde_json::to_value(UiButton::new("Paste from Clipboard", "jwt_paste")).unwrap(),
    ];

    if let Some(err) = &state.jwt.error {
        children.push(
            serde_json::to_value(
                UiText::new(&format!("Error: {}", err))
                    .size(12.0)
                    .content_description("jwt_error"),
            )
            .unwrap(),
        );
    }

    if let Some(h) = &state.jwt.decoded_header {
        children.push(
            serde_json::to_value(
                UiText::new("Header").size(16.0).content_description("jwt_header_title"),
            )
            .unwrap(),
        );
        children.push(
        serde_json::to_value(
            UiCodeView::new(h)
                .language("json")
                .wrap(true)
                .line_numbers(true),
        )
        .unwrap(),
    );
}

    if let Some(p) = &state.jwt.decoded_payload {
        children.push(
            serde_json::to_value(
                UiText::new("Payload").size(16.0).content_description("jwt_payload_title"),
            )
            .unwrap(),
        );
        children.push(
            serde_json::to_value(
                UiCodeView::new(p)
                    .language("json")
                    .wrap(true)
                    .line_numbers(true),
            )
            .unwrap(),
        );
    }

    maybe_push_back(&mut children, state);
    serde_json::to_value(UiColumn::new(children).padding(20)).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_valid_jwt() {
        // HS256 header/payload without signature verification
        let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.signature";
        let (header, payload) = decode_jwt(token).expect("decode should succeed");
        assert!(header.contains("\"alg\""));
        assert!(payload.contains("\"sub\""));
    }

    #[test]
    fn decode_invalid_b64_errors() {
        let token = "not-base64.payload.sig";
        let err = decode_jwt(token).unwrap_err();
        assert!(err.starts_with("jwt_header_b64_error"));
    }
}
