use crate::features::text_viewer::read_text_from_reader;
use crate::state::{AppState, Screen};
use crate::ui::{Button as UiButton, Column as UiColumn, Text as UiText};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::File;
use std::os::unix::io::{FromRawFd, RawFd};
use zip::ZipArchive;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveEntry {
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveState {
    pub path: Option<String>,
    pub entries: Vec<ArchiveEntry>,
    pub error: Option<String>,
    pub truncated: bool,
}

impl ArchiveState {
    pub const fn new() -> Self {
        Self {
            path: None,
            entries: Vec::new(),
            error: None,
            truncated: false,
        }
    }

    pub fn reset(&mut self) {
        self.path = None;
        self.entries.clear();
        self.error = None;
        self.truncated = false;
    }
}

pub fn handle_archive_open(
    state: &mut AppState,
    fd: RawFd,
    path: Option<&str>,
) -> Result<(), String> {
    state.archive.reset();
    state.archive.path = path.map(|s| s.to_string());

    let file = unsafe { File::from_raw_fd(fd) };
    let mut archive = ZipArchive::new(file).map_err(|e| format!("archive_open_failed:{e}"))?;

    let mut entries = Vec::new();
    let limit = 500.min(archive.len());
    for i in 0..limit {
        if let Ok(file) = archive.by_index(i) {
            entries.push(ArchiveEntry {
                name: file.name().to_string(),
                size: file.size(),
                is_dir: file.name().ends_with('/'),
            });
        }
    }
    state.archive.entries = entries;
    state.archive.truncated = archive.len() > limit;
    state.archive.error = None;
    state.replace_current(Screen::ArchiveTools);
    Ok(())
}

pub fn render_archive_screen(state: &AppState) -> Value {
    let mut children = vec![
        serde_json::to_value(UiText::new("Archive Viewer").size(20.0)).unwrap(),
        serde_json::to_value(UiText::new("View contents of .zip files.").size(14.0)).unwrap(),
        serde_json::to_value(
            UiButton::new("Open Archive", "archive_open")
                .requires_file_picker(true)
                .content_description("Pick an archive to list"),
        )
        .unwrap(),
    ];

    if let Some(err) = &state.archive.error {
        children.push(
            serde_json::to_value(
                UiText::new(&format!("Error: {}", err))
                    .size(14.0)
                    .content_description("archive_error"),
            )
            .unwrap(),
        );
    }

    if let Some(path) = &state.archive.path {
        children.push(
            serde_json::to_value(UiText::new(&format!("File: {}", path)).size(12.0)).unwrap(),
        );
    }

    if !state.archive.entries.is_empty() {
        children.push(serde_json::to_value(UiText::new("Contents:").size(16.0)).unwrap());
        let mut rows = Vec::new();
        for (idx, entry) in state.archive.entries.iter().enumerate() {
            let icon = if entry.is_dir { "📁" } else { "📄" };
            let size_str = if entry.is_dir {
                String::new()
            } else {
                format!("({})", human_bytes(entry.size))
            };
            rows.push(
                if is_text_entry(entry) {
                    let label = format!("{icon} {} {size_str}", entry.name);
                    let action = format!("archive_open_text:{idx}");
                    serde_json::to_value(
                        UiButton::new(&label, &action).content_description("archive_entry_text"),
                    )
                    .unwrap()
                } else {
                    serde_json::to_value(
                        UiText::new(&format!("{icon} {} {size_str}", entry.name))
                            .size(14.0)
                            .content_description("archive_entry"),
                    )
                    .unwrap()
                },
            );
        }
        children.push(serde_json::to_value(UiColumn::new(rows).padding(8)).unwrap());
        if state.archive.truncated {
            children.push(
                serde_json::to_value(
                    UiText::new("Showing first 500 entries (truncated)")
                        .size(12.0)
                        .content_description("archive_truncated"),
                )
                .unwrap(),
            );
        }
    } else if state.archive.error.is_none() && state.archive.path.is_some() {
        children.push(
            serde_json::to_value(
                UiText::new("No entries found or archive empty.")
                    .size(12.0)
                    .content_description("archive_empty"),
            )
            .unwrap(),
        );
    }

    if state.nav_depth() > 1 {
        children.push(serde_json::to_value(UiButton::new("Back", "back")).unwrap());
    }

    serde_json::to_value(UiColumn::new(children).padding(24)).unwrap()
}

fn human_bytes(b: u64) -> String {
    const KB: f64 = 1024.0;
    if b < 1024 {
        return format!("{} B", b);
    }
    let kb = b as f64 / KB;
    if kb < KB {
        return format!("{:.1} KB", kb);
    }
    let mb = kb / KB;
    if mb < KB {
        return format!("{:.1} MB", mb);
    }
    let gb = mb / KB;
    format!("{:.1} GB", gb)
}

fn is_text_entry(entry: &ArchiveEntry) -> bool {
    if entry.is_dir {
        return false;
    }
    let name = entry.name.to_ascii_lowercase();
    const TEXT_EXTENSIONS: [&str; 22] = [
        ".txt", ".csv", ".md", ".log", ".json", ".xml", ".yaml", ".yml", ".ini", ".cfg", ".conf",
        ".properties", ".toml", ".rs", ".c", ".cpp", ".h", ".py", ".java", ".kt", ".sh", ".go",
    ];
    TEXT_EXTENSIONS.iter().any(|ext| name.ends_with(ext))
}

pub fn read_text_entry(state: &AppState, index: usize) -> Result<(String, String), String> {
    let archive_path = state
        .archive
        .path
        .as_ref()
        .ok_or_else(|| "archive_missing_path".to_string())?;
    let entry = state
        .archive
        .entries
        .get(index)
        .ok_or_else(|| "archive_entry_out_of_range".to_string())?;

    if entry.is_dir {
        return Err("archive_entry_is_directory".into());
    }
    if !is_text_entry(entry) {
        return Err("archive_entry_not_text".into());
    }

    let file = File::open(archive_path)
        .map_err(|e| format!("archive_reopen_failed:{e}"))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|e| format!("archive_reopen_failed:{e}"))?;
    let mut entry_file = archive
        .by_index(index)
        .map_err(|e| format!("archive_entry_open_failed:{e}"))?;

    let text = read_text_from_reader(&mut entry_file)?;
    let label = format!("{} ⟂ {}", entry.name, archive_path);
    Ok((label, text))
}
