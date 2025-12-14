use crate::features::storage::output_dir_for;
use crate::state::AppState;
use crate::ui::{maybe_push_back, Button as UiButton, Column as UiColumn, Text as UiText};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::Serialize;
use serde_json::{json, Value};
use std::fs::File;
use std::io::{copy, BufReader, Write};
use std::path::{Path, PathBuf};
use rust_i18n::t;

fn to_value_or_text<T: Serialize>(value: T, context: &str) -> Value {
    serde_json::to_value(value).unwrap_or_else(|e| {
        json!({
            "type": "Text",
            "text": format!("{context}_serialize_error:{e}")
        })
    })
}

pub fn render_compression_screen(state: &AppState) -> Value {
    let mut children = vec![
        to_value_or_text(UiText::new(&t!("compression_gzip_title")).size(20.0), "gzip_title"),
        to_value_or_text(
            UiText::new(&t!("compression_gzip_description")).size(14.0),
            "gzip_subtitle",
        ),
        to_value_or_text(
            UiButton::new(&t!("compression_compress_button"), "gzip_compress")
                .requires_file_picker(true)
                .content_description("gzip_compress_btn"),
            "gzip_compress_btn",
        ),
        to_value_or_text(
            UiButton::new(&t!("compression_decompress_button"), "gzip_decompress")
                .requires_file_picker(true)
                .content_description("gzip_decompress_btn"),
            "gzip_decompress_btn",
        ),
    ];

    if let Some(msg) = &state.compression_status {
        children.push(to_value_or_text(
            UiText::new(msg)
                .size(12.0)
                .content_description("gzip_status"),
            "gzip_status",
        ));
        let has_output_path = msg
            .strip_prefix("Result saved to:")
            .map(str::trim)
            .map(|p| !p.is_empty())
            .unwrap_or(false);
        if has_output_path && state.compression_error.is_none() {
            children.push(to_value_or_text(
                UiButton::new(&t!("compression_save_as_button"), "gzip_save_as").id("gzip_save_as_btn"),
                "gzip_save_as_btn",
            ));
        }
    }

    if let Some(err) = &state.compression_error {
        children.push(to_value_or_text(
            UiText::new(&format!("{}{}", t!("multi_hash_error_prefix"), err))
                .size(12.0)
                .content_description("gzip_error"),
            "gzip_error",
        ));
    }

    maybe_push_back(&mut children, state);

    to_value_or_text(UiColumn::new(children).padding(20), "gzip_root")
}

pub fn gzip_compress(path: &str) -> Result<PathBuf, String> {
    let input = Path::new(path);
    if !input.exists() {
        return Err("gzip_source_missing".into());
    }
    if input.is_dir() {
        return Err("gzip_source_is_directory".into());
    }
    if input.is_symlink() {
        return Err("gzip_source_symlink_not_supported".into());
    }

    let mut out_dir = output_dir_for(Some(path));
    let file_name = input
        .file_name()
        .ok_or_else(|| "gzip_missing_filename".to_string())?
        .to_string_lossy();
    out_dir.push(format!("{file_name}.gz"));

    let mut reader =
        BufReader::new(File::open(input).map_err(|e| format!("gzip_open_failed:{e}"))?);
    let out_file = File::create(&out_dir).map_err(|e| format!("gzip_dest_open_failed:{e}"))?;
    let mut encoder = GzEncoder::new(out_file, Compression::default());
    copy(&mut reader, &mut encoder).map_err(|e| format!("gzip_compress_failed:{e}"))?;
    encoder
        .finish()
        .map_err(|e| format!("gzip_compress_failed:{e}"))?;
    Ok(out_dir)
}

pub fn gzip_decompress(path: &str) -> Result<PathBuf, String> {
    let input = Path::new(path);
    if !input.exists() {
        return Err("gzip_source_missing".into());
    }
    if input.is_dir() {
        return Err("gzip_source_is_directory".into());
    }
    if input.is_symlink() {
        return Err("gzip_source_symlink_not_supported".into());
    }

    let mut out_dir = output_dir_for(Some(path));
    let stem = input
        .file_stem()
        .ok_or_else(|| "gzip_missing_filename".to_string())?
        .to_string_lossy();
    out_dir.push(stem.as_ref());

    let reader = BufReader::new(File::open(input).map_err(|e| format!("gzip_open_failed:{e}"))?);
    let mut decoder = GzDecoder::new(reader);
    let mut out_file = File::create(&out_dir).map_err(|e| format!("gzip_dest_open_failed:{e}"))?;
    copy(&mut decoder, &mut out_file).map_err(|e| format!("gzip_decompress_failed:{e}"))?;
    out_file
        .flush()
        .map_err(|e| format!("gzip_decompress_failed:{e}"))?;
    Ok(out_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn find_action(node: &Value, target: &str) -> bool {
        match node {
            Value::Object(map) => {
                if map
                    .get("action")
                    .and_then(|v| v.as_str())
                    .map(|a| a == target)
                    .unwrap_or(false)
                {
                    return true;
                }
                map.get("children")
                    .and_then(|c| c.as_array())
                    .map(|children| children.iter().any(|child| find_action(child, target)))
                    .unwrap_or(false)
            }
            Value::Array(arr) => arr.iter().any(|child| find_action(child, target)),
            _ => false,
        }
    }

    #[test]
    fn renders_save_as_button_on_success_status() {
        let mut state = AppState::new();
        state.compression_status = Some("Result saved to: /tmp/output.gz".into());

        let ui = render_compression_screen(&state);

        assert!(find_action(&ui, "gzip_save_as"));
    }

    #[test]
    fn gzip_roundtrip_preserves_content() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("sample.txt");
        fs::write(&input_path, b"hello gzip").unwrap();

        let gz_path = gzip_compress(input_path.to_str().unwrap()).expect("compress ok");
        let out_path = gzip_decompress(gz_path.to_str().unwrap()).expect("decompress ok");

        let data = fs::read(out_path).unwrap();
        assert_eq!(data, b"hello gzip");
    }
}
