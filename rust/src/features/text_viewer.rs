use crate::state::AppState;
use std::fs::File;
use std::io::Read;
use std::io::{BufReader};
use std::os::fd::FromRawFd;
use std::os::unix::io::RawFd;

const MAX_BYTES: usize = 256 * 1024; // 256 KiB cap to avoid memory bloat
const HEX_PREVIEW_BYTES: usize = 4 * 1024; // cap for hex preview

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

fn is_binary_sample(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }
    bytes.iter().any(|b| *b == 0)
        || bytes
            .iter()
            .any(|b| *b < 0x09 && *b != b'\n' && *b != b'\r')
}

fn hex_preview(bytes: &[u8]) -> String {
    let mut out = String::new();
    for (idx, chunk) in bytes.chunks(16).enumerate() {
        use std::fmt::Write;
        let _ = write!(&mut out, "{:08x}: ", idx * 16);
        for i in 0..16 {
            if let Some(b) = chunk.get(i) {
                let _ = write!(&mut out, "{:02x} ", b);
            } else {
                out.push_str("   ");
            }
        }
        out.push(' ');
        for b in chunk {
            let ch = if b.is_ascii_graphic() || *b == b' ' {
                *b as char
            } else {
                '.'
            };
            out.push(ch);
        }
        out.push('\n');
    }
    out
}

pub fn load_text_preview_from_reader<R: Read>(
    reader: R,
) -> Result<(Option<String>, Option<String>), String> {
    let mut buf_reader = BufReader::new(reader);
    let mut sample = Vec::new();
    buf_reader
        .by_ref()
        .take(HEX_PREVIEW_BYTES as u64)
        .read_to_end(&mut sample)
        .map_err(|e| format!("read_failed:{e}"))?;

    let binary = is_binary_sample(&sample);
    if binary {
        return Ok((None, Some(hex_preview(&sample))));
    }

    // Try to read more as text up to MAX_BYTES
    let mut rest = Vec::new();
    buf_reader
        .take((MAX_BYTES.saturating_sub(sample.len())) as u64)
        .read_to_end(&mut rest)
        .map_err(|e| format!("read_failed:{e}"))?;
    sample.extend(rest);

    Ok((
        Some(
            String::from_utf8(sample.clone())
                .unwrap_or_else(|_| String::from_utf8_lossy(&sample).to_string()),
        ),
        None,
    ))
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
    handle_text_preview(state, file, path);
}

pub fn load_text_from_path(state: &mut AppState, path: &str) {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            state.text_view_error = Some(format!("open_failed:{e}"));
            state.text_view_content = None;
            state.text_view_path = Some(path.to_string());
            return;
        }
    };

    handle_text_preview(state, file, Some(path));
}

fn handle_text_preview<R: Read>(state: &mut AppState, reader: R, path: Option<&str>) {
    state.text_view_hex_preview = None;
    match load_text_preview_from_reader(reader) {
        Ok((Some(text), _)) => {
            state.text_view_content = Some(text);
            state.text_view_error = None;
        }
        Ok((None, Some(hex))) => {
            state.text_view_hex_preview = Some(hex);
            state.text_view_content = None;
            state.text_view_error = Some("binary_preview".into());
        }
        _ => {
            state.text_view_error = Some("read_failed".into());
            state.text_view_content = None;
        }
    }
    if let Some(p) = path {
        state.text_view_path = Some(p.to_string());
        state.text_view_language = guess_language_from_path(p);
    }
}
