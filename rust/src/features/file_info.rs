use infer::Infer;
use serde::Serialize;
use std::fs::File;
use std::io::{BufReader, Read};
use std::os::unix::fs::MetadataExt;
use std::os::unix::io::{FromRawFd, RawFd};
use crate::state::AppState;
use serde_json::{json, Value};
use crate::ui::{CodeView as UiCodeView, Text as UiText, maybe_push_back};

const HEX_PREVIEW_BYTES: usize = 512;

#[derive(Debug, Clone, Serialize)]
pub struct FileInfoResult {
    pub path: Option<String>,
    pub size_bytes: Option<u64>,
    pub mime: Option<String>,
    pub hex_dump: Option<String>,
    pub is_utf8: Option<bool>,
    pub error: Option<String>,
}

pub fn file_info_from_fd(fd: RawFd) -> FileInfoResult {
    if fd < 0 {
        return FileInfoResult {
            path: None,
            size_bytes: None,
            mime: None,
            hex_dump: None,
            is_utf8: None,
            error: Some("invalid_fd".into()),
        };
    }
    let file = unsafe { File::from_raw_fd(fd) };
    info_from_reader(file)
}

pub fn file_info_from_path(path: &str) -> FileInfoResult {
    match File::open(path) {
        Ok(file) => {
            let mut info = info_from_reader(file);
            info.path = Some(path.to_string());
            info
        }
        Err(e) => FileInfoResult {
            path: Some(path.to_string()),
            size_bytes: None,
            mime: None,
            hex_dump: None,
            is_utf8: None,
            error: Some(format!("open_failed:{e}")),
        },
    }
}

fn info_from_reader(file: File) -> FileInfoResult {
    let metadata = match file.metadata() {
        Ok(m) => m,
        Err(e) => {
            return FileInfoResult {
                path: None,
                size_bytes: None,
                mime: None,
                hex_dump: None,
                is_utf8: None,
                error: Some(format!("metadata_failed:{e}")),
            }
        }
    };

    let mut info = FileInfoResult {
        path: None,
        size_bytes: Some(metadata.size()),
        mime: None,
        hex_dump: None,
        is_utf8: None,
        error: None,
    };

    let mut buf = [0u8; 8192];
    let mut reader = BufReader::new(file);
    let read = match reader.read(&mut buf) {
        Ok(r) => r,
        Err(e) => {
            info.error = Some(format!("read_failed:{e}"));
            return info;
        }
    };

    let header_len = read.min(HEX_PREVIEW_BYTES);
    let header = &buf[..header_len];
    if !header.is_empty() {
        info.hex_dump = Some(format_hex_dump(header));
    }
    info.is_utf8 = Some(is_utf8_sample(header));

    let detector = Infer::new();
    info.mime = detector
        .get(&buf[..read])
        .map(|t| t.mime_type().to_string());
    info
}

fn format_hex_dump(bytes: &[u8]) -> String {
    let mut out = String::new();
    for (line, chunk) in bytes.chunks(16).enumerate() {
        let offset = line * 16;
        out.push_str(&format!("{offset:08x}  "));

        for i in 0..16 {
            if let Some(byte) = chunk.get(i) {
                out.push_str(&format!("{byte:02x} "));
            } else {
                out.push_str("   ");
            }
            if i == 7 {
                out.push(' ');
            }
        }

        out.push_str(" |");
        for byte in chunk {
            let ch = if byte.is_ascii_graphic() || *byte == b' ' {
                *byte as char
            } else {
                '.'
            };
            out.push(ch);
        }
        out.push('|');

        if line + 1 < (bytes.len() + 15) / 16 {
            out.push('\n');
        }
    }
    out
}

fn is_utf8_sample(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes).is_ok()
}

pub fn render_file_info_screen(state: &AppState) -> Value {
    let mut children = vec![
        serde_json::to_value(UiText::new("File Inspector").size(20.0)).unwrap(),
        serde_json::to_value(
            UiText::new("Inspect size, MIME type, and a quick hex preview of the file header.")
                .size(14.0),
        )
        .unwrap(),
        json!({
            "type": "Button",
            "text": "Pick file",
            "action": "file_info",
            "requires_file_picker": true
        }),
    ];

    if let Some(info_json) = &state.last_file_info {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(info_json) {
            if let Some(err) = parsed.get("error").and_then(|e| e.as_str()) {
                children.push(json!({
                    "type": "Text",
                    "text": format!("Error: {err}"),
                    "size": 14.0
                }));
            } else {
                if let Some(path) = parsed.get("path").and_then(|p| p.as_str()) {
                    children.push(json!({
                        "type": "Text",
                        "text": format!("Path: {path}"),
                    }));
                }
                if let Some(size) = parsed.get("size_bytes").and_then(|s| s.as_u64()) {
                    children.push(json!({
                        "type": "Text",
                        "text": format!("Size: {} bytes", size),
                    }));
                }
                if let Some(mime) = parsed.get("mime").and_then(|m| m.as_str()) {
                    children.push(json!({
                        "type": "Text",
                        "text": format!("MIME: {mime}"),
                    }));
                }
                if let Some(is_utf8) = parsed.get("is_utf8").and_then(|v| v.as_bool()) {
                    let status = if is_utf8 {
                        "UTF-8 text detected (first 512 bytes)"
                    } else {
                        "Binary / non-UTF-8 bytes detected"
                    };
                    children.push(json!({
                        "type": "Text",
                        "text": status,
                        "size": 14.0
                    }));
                }
                if let Some(hex) = parsed.get("hex_dump").and_then(|h| h.as_str()) {
                    children.push(json!({
                        "type": "Text",
                        "text": "Hex preview (first 512 bytes):",
                        "size": 14.0
                    }));
                    children.push(
                        serde_json::to_value(
                            UiCodeView::new(hex)
                                .wrap(false)
                                .line_numbers(false),
                        )
                        .unwrap(),
                    );
                }
            }
        }
    }

    maybe_push_back(&mut children, state);

    json!({
        "type": "Column",
        "padding": 24,
        "children": children
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_hex_dump_with_offset_and_ascii() {
        let data = b"ABCDEFGHIJKLMNOP";
        let dump = format_hex_dump(data);
        assert_eq!(
            dump,
            "00000000  41 42 43 44 45 46 47 48  49 4a 4b 4c 4d 4e 4f 50  |ABCDEFGHIJKLMNOP|"
        );
    }

    #[test]
    fn detects_utf8_and_non_utf8_samples() {
        assert!(is_utf8_sample("hello".as_bytes()));
        assert!(!is_utf8_sample(&[0xff, 0xfe, 0xfd]));
    }

    #[test]
    fn formats_hex_dump_partial_last_line() {
        let data = b"0123456789abcdefg"; // 17 bytes
        let dump = format_hex_dump(data);
        let lines: Vec<&str> = dump.lines().collect();
        assert_eq!(lines.len(), 2);
        // Check alignment of the second line (padding)
        // 1 byte ("67" -> 'g') then 15 * 3 spaces
        // 00000010  67                                               |g|
        assert!(lines[1].starts_with("00000010  67"));
        assert!(lines[1].ends_with("|g|"));
    }
}
