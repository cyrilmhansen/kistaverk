use crate::features::storage::output_dir_for;
use crate::state::AppState;
use crate::ui::{maybe_push_back, Button as UiButton, Column as UiColumn, Text as UiText, TextInput as UiTextInput};
use age::secrecy::SecretString;
use age::{Decryptor, Encryptor};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs::File;
use std::io::{copy, BufReader, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultState {
    pub input_path: Option<String>,
    pub password: String,
    pub status: Option<String>,
    pub error: Option<String>,
    pub is_processing: bool,
}

impl VaultState {
    pub const fn new() -> Self {
        Self {
            input_path: None,
            password: String::new(),
            status: None,
            error: None,
            is_processing: false,
        }
    }

    pub fn reset(&mut self) {
        self.input_path = None;
        self.password.clear();
        self.status = None;
        self.error = None;
        self.is_processing = false;
    }
}

fn to_value_or_text<T: Serialize>(value: T, context: &str) -> Value {
    serde_json::to_value(value).unwrap_or_else(|e| {
        json!({
            "type": "Text",
            "text": format!("{context}_serialize_error:{e}")
        })
    })
}

pub fn encrypt_file(path: &str, password: &str) -> Result<PathBuf, String> {
    let input = Path::new(path);
    if !input.exists() {
        return Err("vault_source_missing".into());
    }
    if input.is_dir() {
        return Err("vault_source_is_directory".into());
    }
    if input.is_symlink() {
        return Err("vault_source_symlink_not_supported".into());
    }
    if password.trim().is_empty() {
        return Err("vault_missing_password".into());
    }

    let mut out_dir = output_dir_for(Some(path));
    let file_name = input
        .file_name()
        .ok_or_else(|| "vault_missing_filename".to_string())?
        .to_string_lossy();
    let output_name = if file_name.to_lowercase().ends_with(".age") {
        file_name.to_string()
    } else {
        format!("{file_name}.age")
    };
    out_dir.push(output_name);

    let mut reader =
        BufReader::new(File::open(input).map_err(|e| format!("vault_open_failed:{e}"))?);
    let out_file = File::create(&out_dir).map_err(|e| format!("vault_dest_open_failed:{e}"))?;
    let encryptor =
        Encryptor::with_user_passphrase(SecretString::new(password.to_owned()));
    let mut writer = encryptor
        .wrap_output(out_file)
        .map_err(|e| format!("vault_encrypt_failed:{e}"))?;
    copy(&mut reader, &mut writer).map_err(|e| format!("vault_encrypt_failed:{e}"))?;
    writer
        .finish()
        .map_err(|e| format!("vault_encrypt_failed:{e}"))?;
    Ok(out_dir)
}

pub fn decrypt_file(path: &str, password: &str) -> Result<PathBuf, String> {
    let input = Path::new(path);
    if !input.exists() {
        return Err("vault_source_missing".into());
    }
    if input.is_dir() {
        return Err("vault_source_is_directory".into());
    }
    if input.is_symlink() {
        return Err("vault_source_symlink_not_supported".into());
    }
    if password.trim().is_empty() {
        return Err("vault_missing_password".into());
    }

    let mut out_dir = output_dir_for(Some(path));
    let stem = input
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "vault_output".to_string());
    let out_name = if input
        .extension()
        .map(|ext| ext.eq_ignore_ascii_case("age"))
        .unwrap_or(false)
    {
        stem
    } else {
        format!("{stem}.dec")
    };
    out_dir.push(out_name);

    let reader = BufReader::new(File::open(input).map_err(|e| format!("vault_open_failed:{e}"))?);
    let decryptor =
        Decryptor::new(reader).map_err(|e| format!("vault_decrypt_failed:{e}"))?;
    let passphrase_decryptor = match decryptor {
        Decryptor::Passphrase(d) => d,
        _ => return Err("vault_unsupported_recipient".into()),
    };
    let mut decrypted = passphrase_decryptor
        .decrypt(&SecretString::new(password.to_owned()), None)
        .map_err(|e| format!("vault_decrypt_failed:{e}"))?;

    let mut out_file = File::create(&out_dir).map_err(|e| format!("vault_dest_open_failed:{e}"))?;
    copy(&mut decrypted, &mut out_file).map_err(|e| format!("vault_decrypt_failed:{e}"))?;
    out_file
        .flush()
        .map_err(|e| format!("vault_decrypt_failed:{e}"))?;
    Ok(out_dir)
}

pub fn render_vault_screen(state: &AppState) -> Value {
    let mut children = vec![
        to_value_or_text(UiText::new("🔐 The Vault").size(20.0), "vault_title"),
        to_value_or_text(
            UiText::new("Encrypt or decrypt files using age + passphrase.")
                .size(14.0),
            "vault_subtitle",
        ),
        to_value_or_text(
            UiButton::new("Choose file", "vault_pick")
                .requires_file_picker(true)
                .content_description("vault_pick_btn"),
            "vault_pick_btn",
        ),
    ];

    let path_text = state
        .vault
        .input_path
        .as_deref()
        .unwrap_or("No file selected");
    children.push(to_value_or_text(
        UiText::new(&format!("Path: {path_text}"))
            .size(12.0)
            .content_description("vault_path"),
        "vault_path",
    ));

    children.push(to_value_or_text(
        UiTextInput::new("vault_password")
            .hint("Passphrase")
            .single_line(true)
            .password_mask(true)
            .content_description("vault_password_input"),
        "vault_password_input",
    ));

    let buttons = UiColumn::new(vec![
        to_value_or_text(
            UiButton::new("Encrypt", "vault_encrypt").content_description("vault_encrypt_btn"),
            "vault_encrypt_btn",
        ),
        to_value_or_text(
            UiButton::new("Decrypt", "vault_decrypt").content_description("vault_decrypt_btn"),
            "vault_decrypt_btn",
        ),
    ]);
    children.push(to_value_or_text(buttons, "vault_actions"));

    if state.vault.is_processing {
        children.push(to_value_or_text(
            UiText::new("Working...").size(12.0),
            "vault_processing",
        ));
    }

    if let Some(status) = &state.vault.status {
        children.push(to_value_or_text(
            UiText::new(status)
                .size(12.0)
                .content_description("vault_status"),
            "vault_status",
        ));
        let has_output_path = status
            .strip_prefix("Result saved to:")
            .map(str::trim)
            .map(|p| !p.is_empty())
            .unwrap_or(false);
        if has_output_path && state.vault.error.is_none() {
            children.push(to_value_or_text(
                UiButton::new("Save as…", "vault_save_as").id("vault_save_as_btn"),
                "vault_save_as_btn",
            ));
        }
    }

    if let Some(err) = &state.vault.error {
        children.push(to_value_or_text(
            UiText::new(&format!("Error: {}", err))
                .size(12.0)
                .content_description("vault_error"),
            "vault_error",
        ));
    }

    maybe_push_back(&mut children, state);

    to_value_or_text(UiColumn::new(children).padding(20), "vault_root")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn encrypt_then_decrypt_recovers_content() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("secret.txt");
        fs::write(&input_path, b"vault-content").unwrap();

        let enc_path =
            encrypt_file(input_path.to_str().unwrap(), "s3cret").expect("encrypt ok");
        assert!(enc_path.exists());

        let dec_path =
            decrypt_file(enc_path.to_str().unwrap(), "s3cret").expect("decrypt ok");
        assert!(dec_path.exists());

        let data = fs::read(dec_path).unwrap();
        assert_eq!(data, b"vault-content");
    }

    #[test]
    fn decrypt_with_wrong_password_fails() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("secret.txt");
        fs::write(&input_path, b"vault-content").unwrap();

        let enc_path =
            encrypt_file(input_path.to_str().unwrap(), "correct").expect("encrypt ok");
        let err = decrypt_file(enc_path.to_str().unwrap(), "wrong");
        assert!(err.is_err());
    }

    #[test]
    fn decrypt_random_file_returns_error() {
        let dir = tempdir().unwrap();
        let fake_path = dir.path().join("random.bin");
        fs::write(&fake_path, b"not an age file").unwrap();

        let err = decrypt_file(fake_path.to_str().unwrap(), "pwd");
        assert!(err.is_err());
    }

    #[test]
    fn renders_save_as_when_status_contains_path() {
        let mut state = AppState::new();
        state.vault.status = Some("Result saved to: /tmp/vault.age".into());
        let ui = render_vault_screen(&state);
        fn has_action(node: &Value, action: &str) -> bool {
            match node {
                Value::Object(map) => {
                    if map
                        .get("action")
                        .and_then(|v| v.as_str())
                        .map(|a| a == action)
                        .unwrap_or(false)
                    {
                        return true;
                    }
                    map.get("children")
                        .and_then(|c| c.as_array())
                        .map(|arr| arr.iter().any(|child| has_action(child, action)))
                        .unwrap_or(false)
                }
                Value::Array(arr) => arr.iter().any(|child| has_action(child, action)),
                _ => false,
            }
        }

        assert!(has_action(&ui, "vault_save_as"));
    }

    #[test]
    fn password_input_is_masked_in_ui() {
        let state = AppState::new();
        let ui = render_vault_screen(&state);

        fn find_mask(node: &Value) -> Option<bool> {
            match node {
                Value::Object(map) => {
                    let is_password_input = map
                        .get("content_description")
                        .and_then(|v| v.as_str())
                        .map(|s| s == "vault_password_input")
                        .unwrap_or(false);
                    if is_password_input {
                        return map.get("password_mask").and_then(|v| v.as_bool());
                    }
                    map.get("children")
                        .and_then(|c| c.as_array())
                        .and_then(|arr| arr.iter().find_map(find_mask))
                }
                Value::Array(arr) => arr.iter().find_map(find_mask),
                _ => None,
            }
        }

        assert_eq!(find_mask(&ui), Some(true));
    }
}
