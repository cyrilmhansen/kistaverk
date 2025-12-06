use crate::state::{AppState, MultiHashResults};
use crate::ui::{maybe_push_back, Button as UiButton, Text as UiText, TextInput as UiTextInput};
use blake3::Hasher as Blake3;
use crc32fast::Hasher as Crc32;
use md4::Md4;
use md5::Md5;
use serde_json::{json, Value};
use sha1::Sha1;
use sha2::{digest::Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::os::unix::io::{FromRawFd, RawFd};

#[derive(Debug, Clone, Copy)]
pub enum HashAlgo {
    Sha256,
    Sha1,
    Md5,
    Md4,
    Crc32,
    Blake3,
}

#[allow(dead_code)]
pub fn hash_label(algo: HashAlgo) -> &'static str {
    match algo {
        HashAlgo::Sha256 => "SHA-256",
        HashAlgo::Sha1 => "SHA-1",
        HashAlgo::Md5 => "MD5",
        HashAlgo::Md4 => "MD4",
        HashAlgo::Crc32 => "CRC32",
        HashAlgo::Blake3 => "BLAKE3",
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HashVerifyResult {
    pub reference: String,
    pub computed: String,
    pub matches: bool,
    pub algo: HashAlgo,
}

#[allow(dead_code)]
pub fn handle_hash_action(
    state: &mut AppState,
    fd: Option<i32>,
    path: Option<&str>,
    algo: HashAlgo,
) {
    let source = match fd {
        Some(raw) if raw >= 0 => Some(HashSource::RawFd(raw as RawFd)),
        _ => path.map(HashSource::Path),
    };

    let Some(source) = source else {
        state.last_error = Some("missing_path".into());
        state.last_hash = None;
        return;
    };

    match compute_hash(source, algo) {
        Ok(hash) => {
            state.last_hash = Some(hash);
            state.last_error = None;
            if let Some(reference) = &state.hash_reference {
                let cleaned_ref = reference.trim().to_ascii_lowercase();
                let cleaned_hash = state
                    .last_hash
                    .as_ref()
                    .map(|h| h.trim().to_ascii_lowercase())
                    .unwrap_or_default();
                state.hash_match = Some(cleaned_ref == cleaned_hash);
            } else {
                state.hash_match = None;
            }
        }
        Err(e) => {
            state.last_error = Some(e);
            state.last_hash = None;
            state.hash_match = None;
        }
    }
}

#[allow(dead_code)]
pub fn handle_hash_verify(
    state: &mut AppState,
    fd: Option<i32>,
    path: Option<&str>,
    reference: &str,
    algo: HashAlgo,
) {
    let source = match fd {
        Some(raw) if raw >= 0 => Some(HashSource::RawFd(raw as RawFd)),
        _ => path.map(HashSource::Path),
    };

    let Some(source) = source else {
        state.last_error = Some("missing_path".into());
        state.last_hash = None;
        state.hash_match = None;
        return;
    };

    match compute_hash(source, algo) {
        Ok(hash) => {
            let cleaned_ref = reference.trim().to_ascii_lowercase();
            let cleaned_hash = hash.trim().to_ascii_lowercase();
            let matches = cleaned_ref == cleaned_hash;
            state.hash_reference = Some(reference.to_string());
            state.last_hash = Some(hash.clone());
            state.last_hash_algo = Some(hash_label(algo).into());
            state.hash_match = Some(matches);
            state.last_error = None;
        }
        Err(e) => {
            state.last_error = Some(e);
            state.last_hash = None;
            state.hash_match = None;
        }
    }
}

#[allow(dead_code)]
pub fn handle_multi_hash_action(state: &mut AppState, fd: Option<i32>, path: Option<&str>) {
    let source = match fd {
        Some(raw) if raw >= 0 => Some(HashSource::RawFd(raw as RawFd)),
        _ => path.map(HashSource::Path),
    };

    let Some(source) = source else {
        state.multi_hash_error = Some("missing_path".into());
        state.multi_hash_results = None;
        return;
    };

    let file_path_for_display = path
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Selected file".to_string());

    match compute_all_hashes(source, file_path_for_display) {
        Ok(results) => {
            state.multi_hash_results = Some(results);
            state.multi_hash_error = None;
        }
        Err(e) => {
            state.multi_hash_error = Some(e);
            state.multi_hash_results = None;
        }
    }
}

pub enum HashSource<'a> {
    RawFd(RawFd),
    Path(&'a str),
}

pub fn compute_hash(source: HashSource<'_>, algo: HashAlgo) -> Result<String, String> {
    let file = match source {
        HashSource::RawFd(fd) => unsafe { File::from_raw_fd(fd) },
        HashSource::Path(path) => File::open(path).map_err(|e| format!("open_failed:{e}"))?,
    };
    hash_stream(file, algo)
}

fn hash_stream<R: Read>(reader: R, algo: HashAlgo) -> Result<String, String> {
    let mut reader = BufReader::new(reader);
    let mut buffer = [0u8; 8192];
    match algo {
        HashAlgo::Sha256 => {
            let mut hasher = Sha256::new();
            loop {
                let read = reader
                    .read(&mut buffer)
                    .map_err(|e| format!("read_failed:{e}"))?;
                if read == 0 {
                    break;
                }
                hasher.update(&buffer[..read]);
            }
            Ok(format!("{:x}", hasher.finalize()))
        }
        HashAlgo::Sha1 => {
            let mut hasher = Sha1::new();
            loop {
                let read = reader
                    .read(&mut buffer)
                    .map_err(|e| format!("read_failed:{e}"))?;
                if read == 0 {
                    break;
                }
                hasher.update(&buffer[..read]);
            }
            Ok(format!("{:x}", hasher.finalize()))
        }
        HashAlgo::Md5 => {
            let mut hasher = Md5::new();
            loop {
                let read = reader
                    .read(&mut buffer)
                    .map_err(|e| format!("read_failed:{e}"))?;
                if read == 0 {
                    break;
                }
                hasher.update(&buffer[..read]);
            }
            Ok(format!("{:x}", hasher.finalize()))
        }
        HashAlgo::Md4 => {
            let mut hasher = Md4::new();
            loop {
                let read = reader
                    .read(&mut buffer)
                    .map_err(|e| format!("read_failed:{e}"))?;
                if read == 0 {
                    break;
                }
                hasher.update(&buffer[..read]);
            }
            Ok(format!("{:x}", hasher.finalize()))
        }
        HashAlgo::Crc32 => {
            let mut hasher = Crc32::new();
            loop {
                let read = reader
                    .read(&mut buffer)
                    .map_err(|e| format!("read_failed:{e}"))?;
                if read == 0 {
                    break;
                }
                hasher.update(&buffer[..read]);
            }
            Ok(format!("{:08x}", hasher.finalize()))
        }
        HashAlgo::Blake3 => {
            let mut hasher = Blake3::new();
            loop {
                let read = reader
                    .read(&mut buffer)
                    .map_err(|e| format!("read_failed:{e}"))?;
                if read == 0 {
                    break;
                }
                hasher.update(&buffer[..read]);
            }
            Ok(hasher.finalize().to_hex().to_string())
        }
    }
}

pub fn compute_all_hashes(
    source: HashSource<'_>,
    file_path_for_display: String,
) -> Result<MultiHashResults, String> {
    let file = match source {
        HashSource::RawFd(fd) => unsafe { File::from_raw_fd(fd) },
        HashSource::Path(path) => File::open(path).map_err(|e| format!("open_failed:{e}"))?,
    };
    let mut reader = BufReader::new(file);
    let mut buffer = [0u8; 8192];

    let mut sha256_hasher = Sha256::new();
    let mut sha1_hasher = Sha1::new();
    let mut md5_hasher = Md5::new();
    let mut blake3_hasher = Blake3::new();

    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|e| format!("read_failed:{e}"))?;
        if read == 0 {
            break;
        }
        sha256_hasher.update(&buffer[..read]);
        sha1_hasher.update(&buffer[..read]);
        md5_hasher.update(&buffer[..read]);
        blake3_hasher.update(&buffer[..read]);
    }

    Ok(MultiHashResults {
        md5: format!("{:x}", md5_hasher.finalize()),
        sha1: format!("{:x}", sha1_hasher.finalize()),
        sha256: format!("{:x}", sha256_hasher.finalize()),
        blake3: blake3_hasher.finalize().to_hex().to_string(),
        file_path: file_path_for_display,
    })
}

pub fn render_hash_verify_screen(state: &AppState) -> Value {
    let mut children = vec![
        serde_json::to_value(UiText::new("Hash verify (SHA-256)").size(20.0)).unwrap(),
        serde_json::to_value(
            UiText::new("Paste a reference hash, then pick a file to verify.").size(14.0),
        )
        .unwrap(),
        serde_json::to_value(
            UiButton::new("Copy last hash", "noop")
                .id("copy_last_hash_btn")
                .copy_text(state.last_hash.as_deref().unwrap_or("")),
        )
        .unwrap(),
        serde_json::to_value(
            UiButton::new("Paste from clipboard", "hash_verify_paste")
                .id("hash_verify_paste")
                .content_description("hash_verify_paste"),
        )
        .unwrap(),
        serde_json::to_value(
            UiTextInput::new("hash_reference")
                .hint("Reference hash")
                .text(state.hash_reference.as_deref().unwrap_or_default())
                .single_line(true),
        )
        .unwrap(),
        serde_json::to_value(
            UiButton::new("Pick file and verify", "hash_verify")
                .requires_file_picker(true)
                .id("hash_verify_btn"),
        )
        .unwrap(),
    ];

    if let Some(matches) = state.hash_match {
        let status = if matches { "Match ✅" } else { "Mismatch ❌" };
        children.push(
            serde_json::to_value(
                UiText::new(status)
                    .size(14.0)
                    .content_description("hash_verify_status"),
            )
            .unwrap(),
        );
    }
    if let Some(hash) = &state.last_hash {
        children.push(
            serde_json::to_value(
                UiText::new(&format!(
                    "{}: {}",
                    state
                        .last_hash_algo
                        .clone()
                        .unwrap_or_else(|| "SHA-256".into()),
                    hash
                ))
                .size(12.0),
            )
            .unwrap(),
        );
        children.push(
            serde_json::to_value(
                UiButton::new("Copy computed hash", "hash_verify_copy").copy_text(hash),
            )
            .unwrap(),
        );
    }
    if let Some(err) = &state.last_error {
        children.push(
            serde_json::to_value(UiText::new(&format!("Error: {}", err)).size(12.0)).unwrap(),
        );
    }

    maybe_push_back(&mut children, state);

    json!({
        "type": "Column",
        "padding": 24,
        "children": children
    })
}
