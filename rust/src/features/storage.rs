use std::path::PathBuf;

#[cfg(test)]
use std::sync::{Mutex, OnceLock};

#[cfg(test)]
pub fn test_env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

pub fn parse_file_uri_path(uri: &str) -> Option<PathBuf> {
    if let Some(rest) = uri.strip_prefix("file://") {
        return Some(PathBuf::from(rest));
    }
    if uri.starts_with('/') {
        return Some(PathBuf::from(uri));
    }
    None
}

pub fn preferred_temp_dir() -> PathBuf {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(custom) = std::env::var("KISTAVERK_TEMP_DIR") {
        candidates.push(PathBuf::from(custom));
    }
    if let Ok(tmpdir) = std::env::var("TMPDIR") {
        candidates.push(PathBuf::from(tmpdir));
    }
    candidates.push(PathBuf::from("/data/user/0/aeska.kistaverk/cache"));
    candidates.push(PathBuf::from("/data/data/aeska.kistaverk/cache"));
    candidates.push(std::env::temp_dir());

    for dir in candidates {
        if let Ok(meta) = std::fs::metadata(&dir) {
            if meta.is_dir() {
                return dir;
            }
        }
    }
    std::env::temp_dir()
}

pub fn downloads_dir() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(root) = std::env::var("EXTERNAL_STORAGE") {
        candidates.push(PathBuf::from(root).join("Download"));
    }
    candidates.push(PathBuf::from("/storage/emulated/0/Download"));
    candidates.push(PathBuf::from("/sdcard/Download"));

    for dir in candidates {
        if let Ok(meta) = std::fs::metadata(&dir) {
            if meta.is_dir() {
                return Some(dir);
            }
        }
    }
    None
}

pub fn output_dir_for(source_uri: Option<&str>) -> PathBuf {
    if let Some(uri) = source_uri {
        if let Some(path) = parse_file_uri_path(uri) {
            if let Some(parent) = path.parent() {
                return parent.to_path_buf();
            }
        }
        if uri.starts_with("content://") {
            if let Some(dl) = downloads_dir() {
                return dl;
            }
        }
    }
    preferred_temp_dir()
}
