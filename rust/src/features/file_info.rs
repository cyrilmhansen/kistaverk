use infer::Infer;
use serde::Serialize;
use std::fs::File;
use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::os::unix::io::{FromRawFd, RawFd};
use crate::state::AppState;
use serde_json::{json, Value};
use crate::ui::{Text as UiText, maybe_push_back};

#[derive(Debug, Clone, Serialize)]
pub struct FileInfoResult {
    pub path: Option<String>,
    pub size_bytes: Option<u64>,
    pub mime: Option<String>,
    pub error: Option<String>,
}

pub fn file_info_from_fd(fd: RawFd) -> FileInfoResult {
    if fd < 0 {
        return FileInfoResult {
            path: None,
            size_bytes: None,
            mime: None,
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
                error: Some(format!("metadata_failed:{e}")),
            }
        }
    };

    let mut info = FileInfoResult {
        path: None,
        size_bytes: Some(metadata.size()),
        mime: None,
        error: None,
    };

    let mut buf = [0u8; 8192];
    let mut reader = std::io::BufReader::new(file);
    let read = match reader.read(&mut buf) {
        Ok(r) => r,
        Err(e) => {
            info.error = Some(format!("read_failed:{e}"));
            return info;
        }
    };

    let detector = Infer::new();
    info.mime = detector
        .get(&buf[..read])
        .map(|t| t.mime_type().to_string());
    info
}

pub fn render_file_info_screen(state: &AppState) -> Value {
    let mut children = vec![
        serde_json::to_value(UiText::new("File info").size(20.0)).unwrap(),
        serde_json::to_value(UiText::new("Select a file to see its size and MIME type").size(14.0))
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
