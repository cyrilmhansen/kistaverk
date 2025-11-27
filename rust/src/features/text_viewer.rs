use crate::state::AppState;
use std::fs::File;
use std::io::Read;
use std::os::fd::FromRawFd;
use std::os::unix::io::RawFd;

const MAX_BYTES: usize = 256 * 1024; // 256 KiB cap to avoid memory bloat

pub fn read_text_from_reader<R: Read>(mut reader: R) -> Result<String, String> {
    let mut buf = Vec::new();
    let mut limited = reader.by_ref().take(MAX_BYTES as u64);
    limited
        .read_to_end(&mut buf)
        .map_err(|e| format!("read_failed:{e}"))?;

    Ok(
        String::from_utf8(buf.clone())
            .unwrap_or_else(|_| String::from_utf8_lossy(&buf).to_string()),
    )
}

pub fn guess_language_from_path(path: &str) -> Option<String> {
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".rs") {
        Some("rust".into())
    } else if lower.ends_with(".kt") || lower.ends_with(".kts") || lower.ends_with(".java") {
        Some("kotlin".into())
    } else if lower.ends_with(".json") {
        Some("json".into())
    } else if lower.ends_with(".yml") || lower.ends_with(".yaml") {
        Some("yaml".into())
    } else if lower.ends_with(".toml") {
        Some("toml".into())
    } else if lower.ends_with(".md") || lower.ends_with(".markdown") {
        Some("markdown".into())
    } else if lower.ends_with(".sh") || lower.ends_with(".bash") || lower.ends_with(".zsh") {
        Some("bash".into())
    } else if lower.ends_with(".csv") {
        Some("csv".into())
    } else {
        None
    }
}

pub fn load_text_from_fd(state: &mut AppState, fd: RawFd, path: Option<&str>) {
    let file = unsafe { File::from_raw_fd(fd) };
    state.text_view_language = path.and_then(guess_language_from_path);
    match read_text_from_reader(file) {
        Ok(text) => {
            state.text_view_content = Some(text);
            state.text_view_error = None;
            if let Some(p) = path {
                state.text_view_path = Some(p.to_string());
                state.text_view_language = guess_language_from_path(p);
            }
        }
        Err(e) => {
            state.text_view_error = Some(e);
            state.text_view_content = None;
            if let Some(p) = path {
                state.text_view_path = Some(p.to_string());
                state.text_view_language = guess_language_from_path(p);
            }
        }
    }
}

pub fn load_text_from_path(state: &mut AppState, path: &str) {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            state.text_view_error = Some(format!("open_failed:{e}"));
            state.text_view_content = None;
            state.text_view_path = Some(path.to_string());
            return;
        }
    };

    match read_text_from_reader(file.by_ref()) {
        Ok(text) => {
            state.text_view_content = Some(text);
            state.text_view_error = None;
            state.text_view_path = Some(path.to_string());
            state.text_view_language = guess_language_from_path(path);
        }
        Err(e) => {
            state.text_view_error = Some(e);
            state.text_view_content = None;
            state.text_view_path = Some(path.to_string());
            state.text_view_language = guess_language_from_path(path);
        }
    }
}
