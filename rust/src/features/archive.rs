use crate::features::storage::output_dir_for;
use crate::features::text_viewer::read_text_from_reader;
use crate::state::AppState;
use crate::ui::{Button as UiButton, Column as UiColumn, Text as UiText, TextInput as UiTextInput};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs::{self, File};
use std::io::{copy, Write};
use std::os::unix::io::{FromRawFd, RawFd};
use std::path::{Component, Path, PathBuf};
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveEntry {
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
    pub original_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveState {
    pub path: Option<String>,
    pub entries: Vec<ArchiveEntry>,
    pub error: Option<String>,
    pub truncated: bool,
    pub last_output: Option<String>,
    pub filter_query: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveOpenResult {
    pub path: Option<String>,
    pub entries: Vec<ArchiveEntry>,
    pub truncated: bool,
}

impl ArchiveState {
    pub const fn new() -> Self {
        Self {
            path: None,
            entries: Vec::new(),
            error: None,
            truncated: false,
            last_output: None,
            filter_query: None,
        }
    }

    pub fn reset(&mut self) {
        self.path = None;
        self.entries.clear();
        self.error = None;
        self.truncated = false;
        self.last_output = None;
        self.filter_query = None;
    }
}

pub fn open_archive_from_fd(fd: RawFd, path: Option<&str>) -> Result<ArchiveOpenResult, String> {
    let file = unsafe { File::from_raw_fd(fd) };
    read_archive_entries(file, path)
}

pub fn open_archive_from_path(path: &str) -> Result<ArchiveOpenResult, String> {
    let file = File::open(path).map_err(|e| format!("archive_open_failed:{e}"))?;
    read_archive_entries(file, Some(path))
}

fn read_archive_entries(
    file: File,
    path: Option<&str>,
) -> Result<ArchiveOpenResult, String> {
    let mut archive = ZipArchive::new(file).map_err(|e| format!("archive_open_failed:{e}"))?;

    let mut entries = Vec::new();
    let limit = 500.min(archive.len());
    for i in 0..limit {
        if let Ok(file) = archive.by_index(i) {
            entries.push(ArchiveEntry {
                name: file.name().to_string(),
                size: file.size(),
                is_dir: file.name().ends_with('/'),
                original_index: i,
            });
        }
    }
    Ok(ArchiveOpenResult {
        path: path.map(|s| s.to_string()),
        entries,
        truncated: archive.len() > limit,
    })
}

pub fn create_archive(source_path: &str) -> Result<PathBuf, String> {
    let source = Path::new(source_path);
    if !source.exists() {
        return Err("archive_source_missing".into());
    }
    if source.is_symlink() {
        return Err("archive_source_symlink_not_supported".into());
    }

    let dest_dir = output_dir_for(Some(source_path));
    fs::create_dir_all(&dest_dir).map_err(|e| format!("archive_dest_create_failed:{e}"))?;
    let base_name = source
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "archive".to_string());
    let dest_path = dest_dir.join(format!("{base_name}.zip"));

    let file = File::create(&dest_path).map_err(|e| format!("archive_dest_open_failed:{e}"))?;
    let mut writer = ZipWriter::new(file);
    let options = FileOptions::default().compression_method(CompressionMethod::Deflated);

    let base = source
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(""));

    if source.is_dir() {
        let rel = rel_path(&base, source)?;
        let dir_name = if rel.is_empty() {
            String::new()
        } else if rel.ends_with('/') {
            rel
        } else {
            format!("{rel}/")
        };
        if !dir_name.is_empty() {
            writer
                .add_directory(&dir_name, options)
                .map_err(|e| format!("archive_write_failed:{e}"))?;
        }
        write_dir(&mut writer, &base, source, options)?;
    } else {
        let rel = rel_path(&base, source)?;
        write_file(&mut writer, source, &rel, options)?;
    }

    writer
        .finish()
        .map_err(|e| format!("archive_write_failed:{e}"))?;
    Ok(dest_path)
}

fn write_dir(
    writer: &mut ZipWriter<File>,
    base: &Path,
    dir: &Path,
    options: FileOptions,
) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|e| format!("archive_read_dir_failed:{e}"))? {
        let entry = entry.map_err(|e| format!("archive_read_dir_failed:{e}"))?;
        let path = entry.path();
        let meta = entry
            .metadata()
            .map_err(|e| format!("archive_metadata_failed:{e}"))?;
        if meta.file_type().is_symlink() {
            return Err("archive_symlink_not_supported".into());
        }
        if meta.is_dir() {
            let rel = rel_path(base, &path)?;
            let dir_name = if rel.ends_with('/') {
                rel
            } else {
                format!("{rel}/")
            };
            writer
                .add_directory(&dir_name, options)
                .map_err(|e| format!("archive_write_failed:{e}"))?;
            write_dir(writer, base, &path, options)?;
        } else if meta.is_file() {
            let rel = rel_path(base, &path)?;
            write_file(writer, &path, &rel, options)?;
        }
    }
    Ok(())
}

fn write_file(
    writer: &mut ZipWriter<File>,
    path: &Path,
    rel: &str,
    options: FileOptions,
) -> Result<(), String> {
    let mut f = File::open(path).map_err(|e| format!("archive_file_open_failed:{e}"))?;
    writer
        .start_file(rel, options)
        .map_err(|e| format!("archive_write_failed:{e}"))?;
    copy(&mut f, writer).map_err(|e| format!("archive_write_failed:{e}"))?;
    Ok(())
}

fn rel_path(base: &Path, path: &Path) -> Result<String, String> {
    let rel = path
        .strip_prefix(base)
        .map_err(|_| "archive_rel_path_failed".to_string())?;
    let mut parts = Vec::new();
    for comp in rel.components() {
        match comp {
            Component::Normal(part) => parts.push(part.to_string_lossy()),
            Component::CurDir => {}
            _ => return Err("archive_invalid_component".into()),
        }
    }
    Ok(parts.join("/"))
}

pub fn render_archive_screen(state: &AppState) -> Value {
    let mut children = vec![
        to_value_or_text(UiText::new("Archive Viewer").size(20.0), "archive_title"),
        to_value_or_text(
            UiText::new("View contents of .zip files and extract items.").size(14.0),
            "archive_subtitle",
        ),
        to_value_or_text(
            UiButton::new("Open Archive", "archive_open")
                .requires_file_picker(true)
                .content_description("Pick an archive to list"),
            "archive_open_btn",
        ),
    ];

    if state.archive.path.is_some() && !state.archive.entries.is_empty() {
        children.push(to_value_or_text(
            UiButton::new("Extract All", "archive_extract_all")
                .content_description("archive_extract_all"),
            "archive_extract_all",
        ));
    }

    if let Some(err) = &state.archive.error {
        children.push(to_value_or_text(
            UiText::new(&format!("Error: {}", err))
                .size(14.0)
                .content_description("archive_error"),
            "archive_error",
        ));
    }

    if let Some(path) = &state.archive.path {
        children.push(to_value_or_text(
            UiText::new(&format!("File: {}", path)).size(12.0),
            "archive_path",
        ));
    }
    if let Some(msg) = &state.archive.last_output {
        children.push(to_value_or_text(
            UiText::new(msg)
                .size(12.0)
                .content_description("archive_status"),
            "archive_status",
        ));
    }

    if !state.archive.entries.is_empty() {
        let current_filter = state
            .archive
            .filter_query
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("");
        children.push(to_value_or_text(
            UiTextInput::new("archive_filter")
                .hint("Filter entries")
                .text(current_filter)
                .debounce_ms(200)
                .action_on_submit("archive_filter"),
            "archive_filter_input",
        ));
        children.push(to_value_or_text(
            UiText::new("Contents:").size(16.0),
            "archive_contents",
        ));
        let filter = state
            .archive
            .filter_query
            .as_deref()
            .map(|s| s.to_ascii_lowercase());
        let mut rows = Vec::new();
        for entry in state.archive.entries.iter() {
            if let Some(fq) = &filter {
                if !entry.name.to_ascii_lowercase().contains(fq) {
                    continue;
                }
            }
            let icon = if entry.is_dir { "📁" } else { "📄" };
            let size_str = if entry.is_dir {
                String::new()
            } else {
                format!("({})", human_bytes(entry.size))
            };
            let label = format!("{icon} {} {size_str}", entry.name);
            let mut entry_children = Vec::new();
            if is_text_entry(entry) {
                let action = format!("archive_open_text:{}", entry.original_index);
                entry_children.push(to_value_or_text(
                    UiButton::new(&label, &action).content_description("archive_entry_text"),
                    "archive_entry_text",
                ));
            } else {
                entry_children.push(to_value_or_text(
                    UiText::new(&label)
                        .size(14.0)
                        .content_description("archive_entry"),
                    "archive_entry_label",
                ));
            }
            entry_children.push(to_value_or_text(
                UiButton::new("Extract", &format!("archive_extract_entry:{}", entry.original_index))
                    .content_description("archive_extract_entry"),
                "archive_extract_entry",
            ));
            rows.push(to_value_or_text(
                UiColumn::new(entry_children).padding(8),
                "archive_entry_row",
            ));
        }
        children.push(to_value_or_text(
            UiColumn::new(rows).padding(8),
            "archive_entry_list",
        ));
        if state.archive.truncated {
            children.push(to_value_or_text(
                UiText::new("Showing first 500 entries (truncated)")
                    .size(12.0)
                    .content_description("archive_truncated"),
                "archive_truncated",
            ));
        }
    } else if state.archive.error.is_none() && state.archive.path.is_some() {
        children.push(to_value_or_text(
            UiText::new("No entries found or archive empty.")
                .size(12.0)
                .content_description("archive_empty"),
            "archive_empty",
        ));
    }

    if state.nav_depth() > 1 {
        children.push(to_value_or_text(
            UiButton::new("Back", "back"),
            "archive_back",
        ));
    }

    to_value_or_text(UiColumn::new(children).padding(24), "archive_root")
}

fn to_value_or_text<T: Serialize>(value: T, context: &str) -> Value {
    serde_json::to_value(value).unwrap_or_else(|e| {
        json!({
            "type": "Text",
            "text": format!("{context}_serialize_error:{e}")
        })
    })
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
        ".txt",
        ".csv",
        ".md",
        ".log",
        ".json",
        ".xml",
        ".yaml",
        ".yml",
        ".ini",
        ".cfg",
        ".conf",
        ".properties",
        ".toml",
        ".rs",
        ".c",
        ".cpp",
        ".h",
        ".py",
        ".java",
        ".kt",
        ".sh",
        ".go",
    ];
    TEXT_EXTENSIONS.iter().any(|ext| name.ends_with(ext))
}

pub fn read_text_entry(state: &AppState, index: u32) -> Result<(String, String), String> {
    let archive_path = state
        .archive
        .path
        .as_ref()
        .ok_or_else(|| "archive_missing_path".to_string())?;
    let entry = state
        .archive
        .entries
        .get(index as usize)
        .ok_or_else(|| "archive_entry_out_of_range".to_string())?;

    if entry.is_dir {
        return Err("archive_entry_is_directory".into());
    }
    if !is_text_entry(entry) {
        return Err("archive_entry_not_text".into());
    }

    let file = File::open(archive_path).map_err(|e| format!("archive_reopen_failed:{e}"))?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("archive_reopen_failed:{e}"))?;
    let mut entry_file = archive
        .by_index(index as usize)
        .map_err(|e| format!("archive_entry_open_failed:{e}"))?;

    let text = read_text_from_reader(&mut entry_file)?;
    let label = format!("{} ⟂ {}", entry.name, archive_path);
    Ok((label, text))
}

pub fn extract_all(archive_path: &str, dest_root: &Path) -> Result<usize, String> {
    fs::create_dir_all(dest_root).map_err(|e| format!("create_dest_failed:{e}"))?;
    let file = File::open(archive_path).map_err(|e| format!("archive_reopen_failed:{e}"))?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("archive_reopen_failed:{e}"))?;
    let mut count = 0;
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("archive_entry_open_failed:{e}"))?;
        let out_path = safe_join(dest_root, entry.name())?;
        if entry.name().ends_with('/') || entry.is_dir() {
            fs::create_dir_all(&out_path).map_err(|e| format!("create_dir_failed:{e}"))?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).map_err(|e| format!("create_dir_failed:{e}"))?;
            }
            let mut outfile =
                File::create(&out_path).map_err(|e| format!("create_file_failed:{e}"))?;
            copy(&mut entry, &mut outfile).map_err(|e| format!("extract_failed:{e}"))?;
            outfile.flush().map_err(|e| format!("flush_failed:{e}"))?;
        }
        count += 1;
    }
    Ok(count)
}

pub fn extract_entry(archive_path: &str, dest_root: &Path, index: u32) -> Result<PathBuf, String> {
    fs::create_dir_all(dest_root).map_err(|e| format!("create_dest_failed:{e}"))?;
    let file = File::open(archive_path).map_err(|e| format!("archive_reopen_failed:{e}"))?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("archive_reopen_failed:{e}"))?;
    let index_usize = index as usize;
    if index_usize >= archive.len() {
        return Err("archive_entry_out_of_range".into());
    }
    let mut entry = archive
        .by_index(index_usize)
        .map_err(|e| format!("archive_entry_open_failed:{e}"))?;
    let out_path = safe_join(dest_root, entry.name())?;
    if entry.name().ends_with('/') || entry.is_dir() {
        fs::create_dir_all(&out_path).map_err(|e| format!("create_dir_failed:{e}"))?;
    } else {
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("create_dir_failed:{e}"))?;
        }
        let mut outfile = File::create(&out_path).map_err(|e| format!("create_file_failed:{e}"))?;
        copy(&mut entry, &mut outfile).map_err(|e| format!("extract_failed:{e}"))?;
        outfile.flush().map_err(|e| format!("flush_failed:{e}"))?;
    }
    Ok(out_path)
}

fn safe_join(base: &Path, entry_name: &str) -> Result<PathBuf, String> {
    let mut out = PathBuf::from(base);
    let path = Path::new(entry_name);
    for comp in path.components() {
        match comp {
            Component::Normal(part) => out.push(part),
            Component::CurDir => {}
            _ => return Err("invalid_entry_path".into()),
        }
    }
    if !out.starts_with(base) {
        return Err("invalid_entry_path".into());
    }
    Ok(out)
}

pub fn archive_output_root(path: &str) -> PathBuf {
    let base = output_dir_for(Some(path));
    let archive_name = Path::new(path)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "archive".to_string());
    base.join(format!("{}_extracted", archive_name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use tempfile::tempdir;
    use zip::write::FileOptions;

    #[test]
    fn safe_join_rejects_traversal() {
        let base = Path::new("/tmp/base");
        assert!(safe_join(base, "../evil").is_err());
        assert!(safe_join(base, "/abs/path").is_err());
    }

    #[test]
    fn safe_join_allows_nested_paths() {
        let base = Path::new("/tmp/base");
        let out = safe_join(base, "dir/file.txt").unwrap();
        assert!(out.starts_with(base));
        assert!(out.ends_with(Path::new("dir/file.txt")));
    }

    #[test]
    fn extract_all_rejects_traversal_entries() {
        let dir = tempdir().unwrap();
        let zip_path = dir.path().join("test.zip");
        {
            let file = File::create(&zip_path).unwrap();
            let mut writer = zip::ZipWriter::new(file);
            writer
                .start_file("../evil.txt", FileOptions::default())
                .unwrap();
            writer.write_all(b"bad").unwrap();
            writer.finish().unwrap();
        }

        let dest = dir.path().join("out");
        let res = extract_all(zip_path.to_str().unwrap(), &dest);
        assert!(res.is_err());
        assert!(!dest.join("evil.txt").exists());
    }

    #[test]
    fn create_archive_preserves_structure() {
        let dir = tempdir().unwrap();
        let root = dir.path().join("root");
        let sub = root.join("sub");
        fs::create_dir_all(&sub).unwrap();
        fs::write(root.join("a.txt"), b"a").unwrap();
        fs::write(sub.join("b.txt"), b"b").unwrap();

        let out = create_archive(root.to_str().unwrap()).expect("archive created");
        let file = File::open(out).unwrap();
        let mut zip = ZipArchive::new(file).unwrap();
        let mut names: Vec<String> = (0..zip.len())
            .filter_map(|i| zip.by_index(i).ok().map(|f| f.name().to_string()))
            .collect();
        names.sort();
        assert!(names.contains(&"root/".to_string()));
        assert!(names.contains(&"root/a.txt".to_string()));
        assert!(names.contains(&"root/sub/".to_string()));
        assert!(names.contains(&"root/sub/b.txt".to_string()));
    }

    #[test]
    fn create_archive_from_single_file_uses_flat_name() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("single.txt");
        fs::write(&file_path, b"hello").unwrap();

        let out = create_archive(file_path.to_str().unwrap()).expect("archive created");
        let file = File::open(out).unwrap();
        let mut zip = ZipArchive::new(file).unwrap();
        let names: Vec<String> = (0..zip.len())
            .filter_map(|i| zip.by_index(i).ok().map(|f| f.name().to_string()))
            .collect();
        assert_eq!(names, vec!["single.txt".to_string()]);
    }

    #[test]
    fn render_applies_filter_and_preserves_indices() {
        let mut state = AppState::new();
        state.archive.path = Some("archive.zip".into());
        state.archive.entries = vec![
            ArchiveEntry {
                name: "foo.txt".into(),
                size: 10,
                is_dir: false,
                original_index: 0,
            },
            ArchiveEntry {
                name: "logs/output.log".into(),
                size: 100,
                is_dir: false,
                original_index: 5,
            },
        ];
        state.archive.filter_query = Some("log".into());

        let ui = render_archive_screen(&state);
        let ui_str = ui.to_string();
        assert!(
            ui_str.contains("output.log"),
            "filtered entry should be present"
        );
        assert!(
            !ui_str.contains("foo.txt"),
            "non-matching entry should be hidden"
        );
        assert!(
            ui_str.contains("archive_extract_entry:5"),
            "original index should be preserved in actions"
        );
    }
}
