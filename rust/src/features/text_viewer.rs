use crate::state::AppState;
use crate::ui::{
    format_bytes, maybe_push_back, Button as UiButton, CodeView as UiCodeView, Column as UiColumn,
    Text as UiText,
};
use serde_json::{json, Value};
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::os::fd::FromRawFd;
use std::os::unix::io::RawFd;
use std::path::{Path, PathBuf};
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

#[derive(Debug, Clone)]
pub enum TextViewSource {
    Fd {
        fd: RawFd,
        display_path: Option<String>,
    },
    Path {
        read_path: String,
        display_path: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub struct TextViewLoadResult {
    pub content: Option<String>,
    pub hex_preview: Option<String>,
    pub error: Option<String>,
    pub path: Option<String>,
    pub cached_path: Option<String>,
    pub language: Option<String>,
    pub total_bytes: Option<u64>,
    pub loaded_bytes: u64,
    pub window_offset: u64,
    pub has_more: bool,
    pub has_previous: bool,
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

pub fn load_text_for_worker(
    source: TextViewSource,
    offset: u64,
    force_text: bool,
    can_page: bool,
) -> Result<TextViewLoadResult, String> {
    match source {
        TextViewSource::Fd { fd, display_path } => {
            if fd < 0 {
                return Err("invalid_fd".into());
            }
            let mut file = unsafe { File::from_raw_fd(fd) };
            let use_temp = display_path
                .as_deref()
                .map(|p| p.starts_with("content://"))
                .unwrap_or(true);
            if use_temp {
                let temp = copy_fd_to_temp(&mut file)?;
                let res = load_from_path_internal(
                    &temp,
                    display_path.as_deref().or(Some(&temp)),
                    offset,
                    force_text,
                    can_page,
                )?;
                Ok(TextViewLoadResult {
                    cached_path: Some(temp),
                    ..res
                })
            } else {
                let display = display_path.clone().unwrap_or_else(|| "<fd>".into());
                load_from_path_internal(&display, Some(&display), offset, force_text, can_page)
            }
        }
        TextViewSource::Path {
            read_path,
            display_path,
        } => load_from_path_internal(
            &read_path,
            display_path.as_deref(),
            offset,
            force_text,
            can_page,
        ),
    }
}

fn load_from_path_internal(
    path_for_read: &str,
    display_path: Option<&str>,
    offset: u64,
    force_text: bool,
    can_page: bool,
) -> Result<TextViewLoadResult, String> {
    let file = File::open(path_for_read).map_err(|e| format!("open_failed:{e}"))?;
    let total_bytes = file.metadata().ok().map(|m| m.len());
    build_result_from_reader(
        file,
        path_for_read,
        display_path,
        total_bytes,
        offset,
        force_text,
        can_page,
    )
}

fn build_result_from_reader<R: Read>(
    reader: R,
    path_for_read: &str,
    display_path: Option<&str>,
    total_bytes: Option<u64>,
    offset: u64,
    force_text: bool,
    can_page: bool,
) -> Result<TextViewLoadResult, String> {
    let sniff_binary = offset == 0 && !force_text;
    match read_chunk(reader, sniff_binary) {
        Ok(chunk) => {
            let has_content = chunk.content.is_some();
            let path_val = display_path.unwrap_or(path_for_read).to_string();
            let language = display_path.and_then(guess_language_from_path);
            let cached_path = if display_path == Some(path_for_read) {
                None
            } else {
                Some(path_for_read.to_string())
            };

            if let Some(hex) = chunk.hex_preview {
                return Ok(TextViewLoadResult {
                    content: None,
                    hex_preview: Some(hex),
                    error: Some("binary_preview".into()),
                    path: Some(path_val),
                    cached_path,
                    language,
                    total_bytes,
                    loaded_bytes: chunk.bytes_read as u64,
                    window_offset: offset,
                    has_more: false,
                    has_previous: false,
                });
            }

            let loaded_bytes = offset.saturating_add(chunk.bytes_read as u64);
            let eof_known = total_bytes
                .map(|total| loaded_bytes >= total)
                .unwrap_or(chunk.reached_eof);

            Ok(TextViewLoadResult {
                content: chunk.content,
                hex_preview: None,
                error: None,
                path: Some(path_val),
                cached_path,
                language,
                total_bytes,
                loaded_bytes,
                window_offset: offset,
                has_more: can_page && has_content && !eof_known && chunk.bytes_read > 0,
                has_previous: can_page && offset > 0,
            })
        }
        Err(e) => Err(e),
    }
}

pub fn apply_text_view_result(state: &mut AppState, result: TextViewLoadResult) {
    state.text_view_content = result.content;
    state.text_view_hex_preview = result.hex_preview;
    state.text_view_error = result.error;
    state.text_view_window_offset = result.window_offset;
    state.text_view_loaded_bytes = result.loaded_bytes;
    state.text_view_total_bytes = result.total_bytes;
    state.text_view_has_more = result.has_more;
    state.text_view_has_previous = result.has_previous;
    if let Some(path) = result.path {
        state.text_view_path = Some(path.clone());
        if let Some(lang) = result.language {
            state.text_view_language = Some(lang);
        } else {
            state.text_view_language = guess_language_from_path(&path);
        }
    }
    if let Some(cached) = result.cached_path {
        state.text_view_cached_path = Some(cached);
    }
}

fn copy_fd_to_temp(file: &mut File) -> Result<String, String> {
    // Try app-writable temp locations before falling back to std temp.
    let candidates = temp_dirs();
    let mut last_err = None;

    for dir in candidates {
        match std::fs::create_dir_all(&dir) {
            Ok(_) => {}
            Err(e) => {
                last_err = Some(format!("temp_dir_create_failed:{e}"));
                continue;
            }
        }
        match NamedTempFile::new_in(&dir) {
            Ok(mut tmp) => {
                if let Err(e) = file.seek(SeekFrom::Start(0)) {
                    last_err = Some(format!("seek_failed:{e}"));
                    continue;
                }
                if let Err(e) = std::io::copy(file, &mut tmp) {
                    last_err = Some(format!("copy_failed:{e}"));
                    continue;
                }
                match tmp.into_temp_path().keep() {
                    Ok(path) => {
                        if let Some(p) = path.to_str() {
                            return Ok(p.to_string());
                        } else {
                            last_err = Some("temp_path_invalid_utf8".into());
                        }
                    }
                    Err(e) => {
                        last_err = Some(format!("temp_keep_failed:{e}"));
                    }
                }
            }
            Err(e) => {
                last_err = Some(format!("open_failed:{e}"));
            }
        }
    }

    Err(last_err.unwrap_or_else(|| "temp_unavailable".into()))
}

fn temp_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![std::env::temp_dir()];
    let pkg_dirs = [
        "/data/data/aeska.kistaverk/cache",
        "/data/user/0/aeska.kistaverk/cache",
    ];
    for d in pkg_dirs {
        dirs.push(Path::new(d).to_path_buf());
    }
    dirs
}

pub fn render_text_viewer_screen(state: &AppState) -> Value {
    let mut children = vec![
        serde_json::to_value(UiText::new("Text viewer").size(20.0)).unwrap(),
        serde_json::to_value(
            UiText::new(
                "Open a text/CSV/log file and preview it in 128 KB chunks with syntax highlighting."
            )
            .size(14.0),
        )
        .unwrap(),
        json!({
            "type": "Button",
            "text": "Pick text file",
            "action": "text_viewer_open",
            "requires_file_picker": true,
            "content_description": "Pick text or CSV file"
        }),
    ];

    if let Some(path) = &state.text_view_path {
        children.push(
            serde_json::to_value(UiText::new(&format!("File: {}", path)).size(12.0)).unwrap(),
        );
    }

    if state.text_view_loaded_bytes > 0 || state.text_view_total_bytes.is_some() {
        let start = state.text_view_window_offset;
        let end = state.text_view_loaded_bytes;
        let window_size = end.saturating_sub(start);
        let status = if let Some(total) = state.text_view_total_bytes {
            let pct = if total > 0 {
                (end as f64 / total as f64 * 100.0).min(100.0)
            } else {
                100.0
            };
            format!(
                "Showing {}–{} of {} ({} window, {:.1}%)",
                format_bytes(start),
                format_bytes(end),
                format_bytes(total),
                format_bytes(window_size),
                pct
            )
        } else {
            format!(
                "Showing {}–{} ({} window, chunked preview)",
                format_bytes(start),
                format_bytes(end),
                format_bytes(window_size)
            )
        };
        children.push(serde_json::to_value(UiText::new(&status).size(12.0)).unwrap());
    }

    if state.text_view_has_previous || state.text_view_has_more {
        children.push(
            serde_json::to_value(
                json!({
                    "type": "Grid",
                    "columns": 2,
                    "padding": 4,
                    "children": [
                        { "type": "Button", "text": "Load previous", "action": "text_viewer_load_prev", "id": "text_viewer_load_prev", "content_description": "text_viewer_load_prev" },
                        { "type": "Button", "text": "Load next", "action": "text_viewer_load_more", "id": "text_viewer_load_more", "content_description": "text_viewer_load_more" }
                    ]
                })
            )
            .unwrap(),
        );
    }

    children.push(
        serde_json::to_value(json!({
            "type": "Grid",
            "columns": 2,
            "padding": 4,
            "children": [
                {
                    "type": "TextInput",
                    "bind_key": "offset_bytes",
                    "hint": "Byte offset (0 = start)",
                    "text": state.text_view_window_offset.to_string(),
                    "single_line": true,
                    "action_on_submit": "text_viewer_jump"
                },
                {
                    "type": "Button",
                    "text": "Jump",
                    "action": "text_viewer_jump",
                    "content_description": "text_viewer_jump"
                }
            ]
        }))
        .unwrap(),
    );

    // Find bar
    children.push(
        serde_json::to_value(
            UiColumn::new(vec![
                serde_json::to_value(UiText::new("Find in text").size(14.0)).unwrap(),
                serde_json::to_value(
                    UiColumn::new(vec![
                        json!({
                            "type": "TextInput",
                            "bind_key": "find_query",
                            "text": state
                                .text_view_find_query
                                .as_deref()
                                .unwrap_or(""),
                            "hint": "Enter search term",
                            "debounce_ms": 150,
                            "action_on_submit": "text_viewer_find_submit",
                            "single_line": true
                        }),
                        json!({
                            "type": "Grid",
                            "columns": 3,
                            "children": [
                                { "type": "Button", "text": "Prev", "action": "text_viewer_find_prev", "id": "find_prev", "content_description": "find_prev" },
                                { "type": "Button", "text": "Next", "action": "text_viewer_find_next", "id": "find_next", "content_description": "find_next" },
                                { "type": "Button", "text": "Clear", "action": "text_viewer_find_clear", "id": "find_clear", "content_description": "find_clear" }
                            ]
                        }),
                    ])
                    .padding(4),
                )
                .unwrap(),
                serde_json::to_value(
                    UiText::new(
                        state
                            .text_view_find_match
                            .as_deref()
                            .unwrap_or("Type a query and tap next/prev."),
                    )
                    .id("find_status")
                    .size(12.0),
                )
                .unwrap(),
            ])
            .padding(8),
        )
        .unwrap(),
    );

    let theme_label = if state.text_view_dark {
        "Switch to light"
    } else {
        "Switch to dark"
    };
    children.push(
        serde_json::to_value(
            UiButton::new(theme_label, "text_viewer_toggle_theme")
                .content_description("text_viewer_toggle_theme"),
        )
        .unwrap(),
    );
    let ln_label = if state.text_view_line_numbers {
        "Hide line numbers"
    } else {
        "Show line numbers"
    };
    children.push(
        serde_json::to_value(
            UiButton::new(ln_label, "text_viewer_toggle_line_numbers")
                .content_description("text_viewer_toggle_line_numbers"),
        )
        .unwrap(),
    );

    if let Some(err) = &state.text_view_error {
        children.push(
            serde_json::to_value(UiText::new(&format!("Error: {}", err)).size(12.0)).unwrap(),
        );
    }

    if let Some(lang) = state.text_view_language.as_deref() {
        children.push(
            serde_json::to_value(
                UiText::new(&format!("Language: {}", lang))
                    .size(12.0)
                    .content_description("text_viewer_language"),
            )
            .unwrap(),
        );
    }

    if state.text_view_total_bytes.is_some() || state.text_view_loaded_bytes > 0 {
        let total = state
            .text_view_total_bytes
            .map(|v| format!(" / {} bytes", v))
            .unwrap_or_else(|| " / ?".into());
        let loaded = state.text_view_loaded_bytes;
        children.push(
            serde_json::to_value(
                UiText::new(&format!("Loaded: {}{}", loaded, total))
                    .size(12.0)
                    .content_description("text_viewer_progress"),
            )
            .unwrap(),
        );
    }

    if let Some(hex) = &state.text_view_hex_preview {
        children.push(
            serde_json::to_value(
                crate::ui::Warning::new(
                    "Binary or unsupported text detected. Showing hex preview (first 4KB).",
                )
                .content_description("text_viewer_hex_warning"),
            )
            .unwrap(),
        );
        children.push(
            serde_json::to_value(
                UiCodeView::new(hex)
                    .wrap(false)
                    .theme(if state.text_view_dark {
                        "dark"
                    } else {
                        "light"
                    })
                    .line_numbers(false)
                    .id("text_viewer_code_hex"),
            )
            .unwrap(),
        );
        children.push(
            serde_json::to_value(
                UiButton::new("Load anyway (may be slow)", "text_viewer_load_anyway")
                    .content_description("text_viewer_load_anyway"),
            )
            .unwrap(),
        );
    }

    if let Some(content) = &state.text_view_content {
        let mut lang = state.text_view_language.clone();
        if lang.is_none() {
            if let Some(path) = &state.text_view_path {
                lang = guess_language_from_path(path);
            }
        }
        let theme = if state.text_view_dark {
            "dark"
        } else {
            "light"
        };
        let mut code = UiCodeView::new(content)
            .wrap(true)
            .theme(theme)
            .line_numbers(state.text_view_line_numbers)
            .id("text_viewer_code");
        if let Some(lang_str) = lang.as_deref() {
            code = code.language(lang_str);
        }
        children.push(serde_json::to_value(code).unwrap());
        children.push(
            serde_json::to_value(
                UiButton::new("Copy visible text", "noop")
                    .copy_text(content)
                    .id("copy_visible_text"),
            )
            .unwrap(),
        );
    }

    if state.text_view_has_more {
        children.push(
            serde_json::to_value(
                UiButton::new("Load more", "text_viewer_load_more")
                    .id("text_viewer_load_more")
                    .content_description("text_viewer_load_more"),
            )
            .unwrap(),
        );
    }

    maybe_push_back(&mut children, state);

    let mut root = json!({
        "type": "Column",
        "padding": 20,
        "scrollable": false,
        "children": children
    });
    if let Some(q) = &state.text_view_find_query {
        root["find_query"] = json!(q);
    }
    root
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;

    #[test]
    fn find_query_is_exposed_in_render() {
        let mut state = AppState::new();
        state.text_view_find_query = Some("needle".into());
        let ui = render_text_viewer_screen(&state);
        assert_eq!(ui.get("find_query").and_then(|v| v.as_str()), Some("needle"));
    }

    #[test]
    fn code_view_has_stable_id() {
        let mut state = AppState::new();
        state.text_view_content = Some("hello".into());
        let ui = render_text_viewer_screen(&state);
        let children = ui.get("children").and_then(|c| c.as_array()).unwrap();
        let has_code_id = children.iter().any(|child| {
            child
                .get("id")
                .and_then(|v| v.as_str())
                .map(|id| id == "text_viewer_code")
                .unwrap_or(false)
        });
        assert!(has_code_id, "expected CodeView to carry id");
    }
}
