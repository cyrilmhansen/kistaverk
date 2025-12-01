use crate::features::storage::output_dir_for;
use crate::state::AppState;
use crate::ui::{Button as UiButton, Column as UiColumn, Text as UiText, maybe_push_back};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde_json::Value;
use std::fs::File;
use std::io::{copy, BufReader, Write};
use std::path::{Path, PathBuf};

pub fn render_compression_screen(state: &AppState) -> Value {
    let mut children = vec![
        serde_json::to_value(UiText::new("GZIP Compression").size(20.0)).unwrap(),
        serde_json::to_value(
            UiText::new("Compress or decompress single files using .gz.")
                .size(14.0),
        )
        .unwrap(),
        serde_json::to_value(
            UiButton::new("Compress to .gz", "gzip_compress")
                .requires_file_picker(true)
                .content_description("gzip_compress_btn"),
        )
        .unwrap(),
        serde_json::to_value(
            UiButton::new("Decompress .gz", "gzip_decompress")
                .requires_file_picker(true)
                .content_description("gzip_decompress_btn"),
        )
        .unwrap(),
    ];

    if let Some(msg) = &state.compression_status {
        children.push(
            serde_json::to_value(UiText::new(msg).size(12.0).content_description("gzip_status"))
                .unwrap(),
        );
    }

    if let Some(err) = &state.compression_error {
        children.push(
            serde_json::to_value(
                UiText::new(&format!("Error: {}", err))
                    .size(12.0)
                    .content_description("gzip_error"),
            )
            .unwrap(),
        );
    }

    maybe_push_back(&mut children, state);

    serde_json::to_value(UiColumn::new(children).padding(20)).unwrap()
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
    let out_file =
        File::create(&out_dir).map_err(|e| format!("gzip_dest_open_failed:{e}"))?;
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
    let mut out_file =
        File::create(&out_dir).map_err(|e| format!("gzip_dest_open_failed:{e}"))?;
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
