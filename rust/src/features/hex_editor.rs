use crate::features::storage::preferred_temp_dir;
use crate::state::AppState;
use crate::ui::{
    maybe_push_back, Button as UiButton, CodeView as UiCodeView, Column as UiColumn, Grid as UiGrid,
    Section as UiSection, Text as UiText, TextInput as UiTextInput,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::fd::FromRawFd;
use std::os::unix::io::RawFd;

pub const DEFAULT_CHUNK_SIZE: usize = 256;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HexEditorState {
    pub file_path: Option<String>,
    pub display_path: Option<String>,
    pub offset: u64,
    pub chunk_size: usize,
    pub dirty_bytes: BTreeMap<u64, u8>,
    pub current_dump: Option<String>,
    pub error: Option<String>,
    pub status: Option<String>,
}

impl HexEditorState {
    pub const fn new() -> Self {
        Self {
            file_path: None,
            display_path: None,
            offset: 0,
            chunk_size: DEFAULT_CHUNK_SIZE,
            dirty_bytes: BTreeMap::new(),
            current_dump: None,
            error: None,
            status: None,
        }
    }

    pub fn reset(&mut self) {
        self.file_path = None;
        self.display_path = None;
        self.offset = 0;
        self.chunk_size = DEFAULT_CHUNK_SIZE;
        self.dirty_bytes.clear();
        self.current_dump = None;
        self.error = None;
        self.status = None;
    }
}

pub fn set_file(state: &mut HexEditorState, path: String, display_path: Option<String>) -> Result<(), String> {
    state.file_path = Some(path.clone());
    state.display_path = display_path.or_else(|| Some(path.clone()));
    state.offset = 0;
    state.dirty_bytes.clear();
    state.status = Some("Loaded file".into());
    refresh_view(state)
}

pub fn refresh_view(state: &mut HexEditorState) -> Result<(), String> {
    let path = state
        .file_path
        .as_ref()
        .ok_or_else(|| "no_file_selected".to_string())?;
    let mut file = File::open(path).map_err(|e| format!("open_failed:{e}"))?;
    let meta = file.metadata().map_err(|e| format!("stat_failed:{e}"))?;
    let len = meta.len();

    if len == 0 {
        state.current_dump = Some("Empty file".into());
        state.error = None;
        return Ok(());
    }

    let chunk = state.chunk_size.max(1) as u64;
    let max_start = len.saturating_sub(1).saturating_sub(chunk.saturating_sub(1));
    if state.offset > max_start {
        state.offset = max_start;
    }

    file.seek(SeekFrom::Start(state.offset))
        .map_err(|e| format!("seek_failed:{e}"))?;
    let mut buf = vec![0u8; state.chunk_size];
    let read = file
        .read(&mut buf)
        .map_err(|e| format!("read_failed:{e}"))?;
    buf.truncate(read);

    overlay_dirty(&mut buf, state.offset, &state.dirty_bytes);
    state.current_dump = Some(format_hex_dump(&buf, state.offset, &state.dirty_bytes));
    state.error = None;
    Ok(())
}

pub fn patch_byte(state: &mut HexEditorState, offset: u64, byte: u8) -> Result<(), String> {
    let path = state
        .file_path
        .as_ref()
        .ok_or_else(|| "no_file_selected".to_string())?;
    let meta = File::open(path)
        .and_then(|f| f.metadata())
        .map_err(|e| format!("stat_failed:{e}"))?;
    if offset >= meta.len() {
        return Err("offset_out_of_range".into());
    }
    state.dirty_bytes.insert(offset, byte);
    state.status = Some(format!("Patched 0x{offset:08x} -> 0x{byte:02x}"));
    refresh_view(state)
}

pub fn save_changes(state: &mut HexEditorState) -> Result<String, String> {
    let path = state
        .file_path
        .clone()
        .ok_or_else(|| "no_file_selected".to_string())?;
    if state.dirty_bytes.is_empty() {
        state.status = Some("No changes to save".into());
        state.error = None;
        return Ok(path.clone());
    }
    let mut file = OpenOptions::new()
        .write(true)
        .read(true)
        .open(&path)
        .map_err(|e| format!("open_failed:{e}"))?;
    apply_patches(&mut file, &state.dirty_bytes)?;
    file.sync_all().ok();
    state.dirty_bytes.clear();
    state.status = Some(format!("Result saved to: {path}"));
    state.error = None;
    refresh_view(state)?;
    Ok(path)
}

pub fn save_as(state: &mut HexEditorState, target: &str) -> Result<String, String> {
    if target.trim().is_empty() {
        return Err("save_as_path_missing".into());
    }
    let source = state
        .file_path
        .clone()
        .ok_or_else(|| "no_file_selected".to_string())?;
    let target_path = target.trim();
    if let Some(parent) = std::path::Path::new(target_path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| format!("mkdir_failed:{e}"))?;
        }
    }
    fs::copy(&source, target_path).map_err(|e| format!("copy_failed:{e}"))?;
    let mut file = OpenOptions::new()
        .write(true)
        .read(true)
        .open(target_path)
        .map_err(|e| format!("open_failed:{e}"))?;
    apply_patches(&mut file, &state.dirty_bytes)?;
    file.sync_all().ok();
    state.dirty_bytes.clear();
    state.file_path = Some(target_path.to_string());
    state.display_path = Some(target_path.to_string());
    state.status = Some(format!("Result saved to: {target_path}"));
    state.error = None;
    refresh_view(state)?;
    Ok(target_path.to_string())
}

pub fn export_to_temp(state: &mut HexEditorState) -> Result<String, String> {
    let source = state
        .file_path
        .as_ref()
        .ok_or_else(|| "no_file_selected".to_string())?;
    let dir = preferred_temp_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("temp_dir_failed:{e}"))?;
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let target = dir.join(format!(
        "hex_export_{}_{}",
        ts,
        std::path::Path::new(source)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("file")
    ));
    fs::copy(source, &target).map_err(|e| format!("copy_failed:{e}"))?;
    let mut file = OpenOptions::new()
        .write(true)
        .read(true)
        .open(&target)
        .map_err(|e| format!("open_failed:{e}"))?;
    apply_patches(&mut file, &state.dirty_bytes)?;
    file.sync_all().ok();
    let path_str = target
        .to_str()
        .ok_or_else(|| "temp_path_invalid_utf8".to_string())?
        .to_string();
    state.status = Some(format!("Result saved to: {path_str}"));
    state.error = None;
    refresh_view(state)?;
    Ok(path_str)
}

pub fn copy_fd_to_temp(fd: RawFd) -> Result<String, String> {
    if fd < 0 {
        return Err("invalid_fd".into());
    }
    let mut file = unsafe { File::from_raw_fd(fd) };
    let dir = preferred_temp_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("temp_dir_failed:{e}"))?;
    let mut temp =
        tempfile::NamedTempFile::new_in(&dir).map_err(|e| format!("temp_open_failed:{e}"))?;
    file.seek(SeekFrom::Start(0))
        .map_err(|e| format!("seek_failed:{e}"))?;
    std::io::copy(&mut file, temp.as_file_mut()).map_err(|e| format!("copy_failed:{e}"))?;
    let keep = temp
        .into_temp_path()
        .keep()
        .map_err(|e| format!("temp_keep_failed:{e}"))?;
    keep.to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "temp_path_invalid_utf8".into())
}

fn apply_patches(file: &mut File, dirty: &BTreeMap<u64, u8>) -> Result<(), String> {
    for (offset, byte) in dirty.iter() {
        file.seek(SeekFrom::Start(*offset))
            .map_err(|e| format!("seek_failed:{e}"))?;
        file.write_all(&[*byte])
            .map_err(|e| format!("write_failed:{e}"))?;
    }
    Ok(())
}

fn overlay_dirty(buf: &mut [u8], base: u64, dirty: &BTreeMap<u64, u8>) {
    let end = base.saturating_add(buf.len() as u64);
    for (offset, byte) in dirty.range(base..=end) {
        if let Some(rel) = offset.checked_sub(base) {
            if let Ok(idx) = usize::try_from(rel) {
                if let Some(slot) = buf.get_mut(idx) {
                    *slot = *byte;
                }
            }
        }
    }
}

fn format_hex_dump(bytes: &[u8], start_offset: u64, dirty: &BTreeMap<u64, u8>) -> String {
    let mut out = String::new();
    for (line_idx, chunk) in bytes.chunks(16).enumerate() {
        use std::fmt::Write;
        let absolute = start_offset + (line_idx as u64 * 16);
        let _ = write!(&mut out, "{absolute:08x}: ");
        for i in 0..16 {
            if let Some(b) = chunk.get(i) {
                let byte_offset = absolute + i as u64;
                let marker = if dirty.contains_key(&byte_offset) { '*' } else { ' ' };
                let _ = write!(&mut out, "{b:02x}{marker} ");
            } else {
                out.push_str("    ");
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

pub fn render_hex_editor_screen(state: &AppState) -> Value {
    let editor = &state.hex_editor;
    let mut children = vec![
        serde_json::to_value(UiText::new("Hex / Binary Editor").size(20.0)).unwrap(),
        serde_json::to_value(
            UiText::new(&format!(
                "Path: {}",
                editor
                    .display_path
                    .as_deref()
                    .unwrap_or("No file selected. Pick a file to begin.")
            ))
            .size(12.0),
        )
        .unwrap(),
    ];

    if let Some(status) = &editor.status {
        children.push(serde_json::to_value(UiText::new(status).size(12.0)).unwrap());
    }
    if let Some(err) = &editor.error {
        children.push(
            serde_json::to_value(UiText::new(&format!("Error: {err}")).size(12.0)).unwrap(),
        );
    }
    if !editor.dirty_bytes.is_empty() {
        children.push(
            serde_json::to_value(UiText::new("Patched bytes are marked with * in the hex view.").size(12.0))
                .unwrap(),
        );
    }

    // File controls
    let file_controls = UiGrid::new(vec![
        json!(UiButton::new("Pick file", "hex_editor_open").requires_file_picker(true)),
        json!(UiButton::new("Save", "hex_editor_save")),
        json!(UiButton::new("Save As…", "hex_editor_save_as_picker")),
    ])
    .columns(3)
    .padding(8);
    children.push(serde_json::to_value(file_controls).unwrap());

    // Navigation
    let nav_inputs = UiGrid::new(vec![
        json!(UiButton::new("Prev chunk", "hex_editor_prev")),
        json!(UiButton::new("Next chunk", "hex_editor_next")),
        json!(UiTextInput::new("hex_jump_offset")
            .hint("Jump offset (dec or 0x)")
            .single_line(true)
            .debounce_ms(200)
            .action_on_submit("hex_editor_jump")),
    ])
    .columns(3)
    .padding(8);
    children.push(serde_json::to_value(nav_inputs).unwrap());

    // Patch controls
    let patch_section = UiSection::new(vec![
        json!(UiTextInput::new("hex_patch_offset")
            .hint("Offset (dec or 0x)")
            .single_line(true)
            .debounce_ms(200)),
        json!(UiTextInput::new("hex_patch_value")
            .hint("Byte value (00-ff)")
            .single_line(true)
            .debounce_ms(200)),
        json!(UiButton::new("Patch byte", "hex_editor_patch")),
    ])
    .title("Edit")
    .padding(12);
    children.push(serde_json::to_value(patch_section).unwrap());

    // Viewer
    if let Some(dump) = &editor.current_dump {
        children.push(
            serde_json::to_value(
                UiCodeView::new(dump)
                    .language("none")
                    .wrap(false)
                    .line_numbers(true),
            )
            .unwrap(),
        );
    } else {
        children.push(
            serde_json::to_value(UiText::new("No data loaded yet.").size(12.0)).unwrap(),
        );
    }

    maybe_push_back(&mut children, state);
    serde_json::to_value(UiColumn::new(children).padding(20)).unwrap()
}

fn parse_hex_byte(input: &str) -> Option<u8> {
    let trimmed = input.trim().trim_start_matches("0x").trim_start_matches("0X");
    u8::from_str_radix(trimmed, 16).ok()
}

fn parse_offset(input: &str) -> Option<u64> {
    let trimmed = input.trim();
    if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
        u64::from_str_radix(trimmed.trim_start_matches("0x").trim_start_matches("0X"), 16).ok()
    } else {
        trimmed.parse::<u64>().ok()
    }
}

pub fn parse_offset_binding(bindings: &std::collections::HashMap<String, String>, key: &str) -> Option<u64> {
    bindings.get(key).and_then(|v| parse_offset(v))
}

pub fn parse_byte_binding(bindings: &std::collections::HashMap<String, String>, key: &str) -> Option<u8> {
    bindings.get(key).and_then(|v| parse_hex_byte(v))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn patch_overlay_is_visible_in_dump() {
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(&[0x00, 0x01, 0x02, 0x03]).unwrap();
        let path = tmp.path().to_str().unwrap().to_string();

        let mut state = HexEditorState::new();
        set_file(&mut state, path, None).expect("load");
        patch_byte(&mut state, 1, 0xff).expect("patch");
        let dump = state.current_dump.clone().unwrap();
        assert!(dump.contains("ff*"), "dump was: {dump}");
    }

    #[test]
    fn save_changes_writes_to_disk() {
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(&[0xaa, 0xbb, 0xcc]).unwrap();
        let path = tmp.path().to_str().unwrap().to_string();

        let mut state = HexEditorState::new();
        set_file(&mut state, path.clone(), None).expect("load");
        patch_byte(&mut state, 2, 0x99).expect("patch");
        save_changes(&mut state).expect("save");

        let mut data = Vec::new();
        File::open(path)
            .unwrap()
            .read_to_end(&mut data)
            .unwrap();
        assert_eq!(data, vec![0xaa, 0xbb, 0x99]);
    }

    #[test]
    fn export_to_temp_includes_patches() {
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(&[0x10, 0x20, 0x30]).unwrap();
        let path = tmp.path().to_str().unwrap().to_string();

        let mut state = HexEditorState::new();
        set_file(&mut state, path, None).expect("load");
        patch_byte(&mut state, 0, 0xfe).expect("patch");
        let exported = export_to_temp(&mut state).expect("export");

        let mut data = Vec::new();
        File::open(exported).unwrap().read_to_end(&mut data).unwrap();
        assert_eq!(data, vec![0xfe, 0x20, 0x30]);
    }
}
