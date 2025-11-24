use crate::state::AppState;
use std::fs::File;
use std::io::Read;
use std::os::fd::FromRawFd;
use std::os::unix::io::RawFd;

const MAX_BYTES: usize = 256 * 1024; // 256 KiB cap to avoid memory bloat

pub fn load_text_from_fd(state: &mut AppState, fd: RawFd, path: Option<&str>) {
    let file = unsafe { File::from_raw_fd(fd) };
    let mut buf = Vec::new();
    let mut limited = file.take(MAX_BYTES as u64);
    match limited.read_to_end(&mut buf) {
        Ok(_) => {
            // Try to decode UTF-8; if invalid, fallback to lossy.
            let text = String::from_utf8(buf.clone())
                .unwrap_or_else(|_| String::from_utf8_lossy(&buf).to_string());
            state.text_view_content = Some(text);
            state.text_view_error = None;
            if let Some(p) = path {
                state.text_view_path = Some(p.to_string());
            }
        }
        Err(e) => {
            state.text_view_error = Some(format!("read_failed:{e}"));
            state.text_view_content = None;
            if let Some(p) = path {
                state.text_view_path = Some(p.to_string());
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

    let mut buf = Vec::new();
    let mut limited = file.by_ref().take(MAX_BYTES as u64);
    match limited.read_to_end(&mut buf) {
        Ok(_) => {
            let text = String::from_utf8(buf.clone())
                .unwrap_or_else(|_| String::from_utf8_lossy(&buf).to_string());
            state.text_view_content = Some(text);
            state.text_view_error = None;
            state.text_view_path = Some(path.to_string());
        }
        Err(e) => {
            state.text_view_error = Some(format!("read_failed:{e}"));
            state.text_view_content = None;
            state.text_view_path = Some(path.to_string());
        }
    }
}
