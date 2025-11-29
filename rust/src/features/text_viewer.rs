use crate::state::AppState;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::os::fd::FromRawFd;
use std::os::unix::io::RawFd;
use tempfile::NamedTempFile;

const MAX_BYTES: usize = 256 * 1024; // 256 KiB cap to avoid memory bloat for generic reads
pub const CHUNK_BYTES: usize = 128 * 1024; // chunk size for incremental loads
const HEX_PREVIEW_BYTES: usize = 4 * 1024; // cap for hex preview

pub fn read_text_from_reader<R: Read>(mut reader: R) -> Result<String, String> {
    let mut buf = Vec::new();
    let mut limited = reader.by_ref().take(MAX_BYTES as u64);
    limited
        .read_to_end(&mut buf)
        .map_err(|e| format!("read_failed:{e}"))?;

    Ok(bytes_to_string(buf))
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

#[derive(Debug)]
struct ChunkOutcome {
    content: Option<String>,
    hex_preview: Option<String>,
    bytes_read: usize,
    reached_eof: bool,
}

fn bytes_to_string(buf: Vec<u8>) -> String {
    String::from_utf8(buf.clone()).unwrap_or_else(|_| String::from_utf8_lossy(&buf).to_string())
}

fn read_chunk<R: Read>(reader: R, sniff_binary: bool) -> Result<ChunkOutcome, String> {
    let mut buf_reader = BufReader::new(reader);
    let mut collected = Vec::new();
    let mut total_read = 0usize;

    if sniff_binary {
        let mut sample = Vec::new();
        let sample_limit = HEX_PREVIEW_BYTES.min(CHUNK_BYTES);
        let read = buf_reader
            .by_ref()
            .take(sample_limit as u64)
            .read_to_end(&mut sample)
            .map_err(|e| format!("read_failed:{e}"))?;
        total_read += read;
        if is_binary_sample(&sample) {
            return Ok(ChunkOutcome {
                content: None,
                hex_preview: Some(hex_preview(&sample)),
                bytes_read: total_read,
                reached_eof: read < CHUNK_BYTES,
            });
        }
        collected.extend(sample);
    }

    let remaining = CHUNK_BYTES.saturating_sub(collected.len());
    let mut rest = Vec::new();
    let read = buf_reader
        .take(remaining as u64)
        .read_to_end(&mut rest)
        .map_err(|e| format!("read_failed:{e}"))?;
    total_read += read;
    collected.extend(rest);

    Ok(ChunkOutcome {
        content: Some(bytes_to_string(collected)),
        hex_preview: None,
        bytes_read: total_read,
        reached_eof: total_read < CHUNK_BYTES,
    })
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
    let mut file = unsafe { File::from_raw_fd(fd) };
    let total_bytes = file.metadata().ok().map(|m| m.len());
    state.text_view_language = path.and_then(guess_language_from_path);

    let use_temp = path.is_none() || path.map(|p| p.starts_with("content://")).unwrap_or(false);
    if use_temp {
        match copy_fd_to_temp(&mut file) {
            Ok(temp_path) => {
                state.text_view_cached_path = Some(temp_path.clone());
                handle_text_from_path(state, &temp_path, path.or(Some(temp_path.as_str())), 0, false, true);
            }
            Err(e) => {
                state.text_view_error = Some(e);
                state.text_view_content = None;
            }
        }
    } else {
        state.text_view_cached_path = None;
        handle_text_chunk(state, file, path, total_bytes, 0, false, path.is_some());
    }
}

pub fn load_text_from_path(state: &mut AppState, path: &str) {
    load_text_from_path_at_offset(state, path, 0, false);
}

pub fn load_text_from_path_at_offset(
    state: &mut AppState,
    path: &str,
    offset: u64,
    force_text: bool,
) {
    // Window reads re-open the file each time to keep memory bounded.
    if offset > 0 {
        match File::open(path)
            .and_then(|mut f| f.seek(SeekFrom::Start(offset)).map(|_| ()))
        {
            Ok(_) => {}
            Err(e) => {
                state.text_view_error = Some(format!("seek_failed:{e}"));
                state.text_view_has_more = false;
                return;
            }
        }
    }

    handle_text_from_path(state, path, Some(path), offset, force_text, true);
}

pub fn load_more_text(state: &mut AppState) {
    if state.text_view_hex_preview.is_some() {
        state.text_view_error = Some("binary_preview".into());
        return;
    }
    let path = match state.text_view_path.clone() {
        Some(p) => p,
        None => {
            state.text_view_error = Some("missing_path".into());
            return;
        }
    };
    let offset = state
        .text_view_window_offset
        .saturating_add(CHUNK_BYTES as u64);
    let effective = effective_path(state, &path);
    load_text_from_path_at_offset(state, &effective, offset, true);
}

pub fn load_prev_text(state: &mut AppState) {
    let path = match state.text_view_path.clone() {
        Some(p) => p,
        None => {
            state.text_view_error = Some("missing_path".into());
            return;
        }
    };
    let offset = state
        .text_view_window_offset
        .saturating_sub(CHUNK_BYTES as u64);
    let effective = effective_path(state, &path);
    load_text_from_path_at_offset(state, &effective, offset, true);
}

fn handle_text_from_path(
    state: &mut AppState,
    path_for_read: &str,
    display_path: Option<&str>,
    offset: u64,
    force_text: bool,
    can_page: bool,
) {
    let file = match File::open(path_for_read) {
        Ok(f) => f,
        Err(e) => {
            state.text_view_error = Some(format!("open_failed:{e}"));
            state.text_view_has_more = false;
            state.text_view_has_previous = offset > 0;
            return;
        }
    };
    let total_bytes = file.metadata().ok().map(|m| m.len());
    handle_text_chunk(
        state,
        file,
        display_path,
        total_bytes,
        offset,
        force_text,
        can_page,
    );
}

fn handle_text_chunk<R: Read>(
    state: &mut AppState,
    reader: R,
    path: Option<&str>,
    total_bytes: Option<u64>,
    offset: u64,
    force_text: bool,
    can_page: bool,
) {
    state.text_view_hex_preview = None;
    let sniff_binary = offset == 0 && !force_text;

    match read_chunk(reader, sniff_binary) {
        Ok(chunk) => {
            let has_content = chunk.content.is_some();
            if let Some(hex) = chunk.hex_preview {
                state.text_view_hex_preview = Some(hex);
                state.text_view_content = None;
                state.text_view_error = Some("binary_preview".into());
                state.text_view_has_more = false;
                state.text_view_has_previous = false;
                state.text_view_loaded_bytes = chunk.bytes_read as u64;
                state.text_view_total_bytes = total_bytes;
                return;
            }

            if let Some(text) = chunk.content {
                state.text_view_content = Some(text);
                state.text_view_error = None;
            } else {
                state.text_view_content = None;
                state.text_view_error = Some("read_failed".into());
            }

            state.text_view_window_offset = offset;
            state.text_view_loaded_bytes = offset.saturating_add(chunk.bytes_read as u64);
            state.text_view_total_bytes = total_bytes;
            let eof_known = total_bytes
                .map(|total| state.text_view_loaded_bytes >= total)
                .unwrap_or(chunk.reached_eof);
            state.text_view_has_more =
                can_page && has_content && !eof_known && chunk.bytes_read > 0;
            state.text_view_has_previous = can_page && offset > 0;
        }
        Err(e) => {
            state.text_view_error = Some(e);
            state.text_view_content = None;
            state.text_view_loaded_bytes = offset;
            state.text_view_has_more = false;
            state.text_view_has_previous = offset > 0;
        }
    }

    if let Some(p) = path {
        state.text_view_path = Some(p.to_string());
        state.text_view_language = guess_language_from_path(p);
    }
}

fn effective_path(state: &AppState, primary: &str) -> String {
    state
        .text_view_cached_path
        .clone()
        .unwrap_or_else(|| primary.to_string())
}

fn copy_fd_to_temp(file: &mut File) -> Result<String, String> {
    let mut tmp = NamedTempFile::new().map_err(|e| format!("open_failed:{e}"))?;
    file.seek(SeekFrom::Start(0))
        .map_err(|e| format!("seek_failed:{e}"))?;
    std::io::copy(file, &mut tmp).map_err(|e| format!("copy_failed:{e}"))?;
    let path = tmp
        .into_temp_path()
        .keep()
        .map_err(|e| format!("temp_keep_failed:{e}"))?;
    path.to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "temp_path_invalid_utf8".to_string())
}
