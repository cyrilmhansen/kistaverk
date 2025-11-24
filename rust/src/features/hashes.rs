use crate::state::AppState;
use serde::Deserialize;
use sha2::{Sha256, digest::Digest};
use sha1::Sha1;
use md5::Md5;
use md4::Md4;
use std::fs::File;
use std::io::{BufReader, Read};

#[derive(Debug, Clone, Copy)]
pub enum HashAlgo {
    Sha256,
    Sha1,
    Md5,
    Md4,
}

#[derive(Deserialize)]
pub struct Command {
    pub path: Option<String>,
    pub error: Option<String>,
}

pub fn handle_hash_action(state: &mut AppState, path: Option<&str>, algo: HashAlgo) {
    match path {
        Some(p) => match compute_hash(p, algo) {
            Ok(hash) => {
                state.last_hash = Some(hash);
                state.last_error = None;
            }
            Err(e) => {
                state.last_error = Some(e);
                state.last_hash = None;
            }
        },
        None => {
            state.last_error = Some("missing_path".into());
            state.last_hash = None;
        }
    }
}

fn compute_hash(path: &str, algo: HashAlgo) -> Result<String, String> {
    let file = File::open(path).map_err(|e| format!("open_failed:{e}"))?;
    let mut reader = BufReader::new(file);
    let mut buffer = [0u8; 8192];

    match algo {
        HashAlgo::Sha256 => {
            let mut hasher = Sha256::new();
            loop {
                let read = reader.read(&mut buffer).map_err(|e| format!("read_failed:{e}"))?;
                if read == 0 { break; }
                hasher.update(&buffer[..read]);
            }
            Ok(format!("{:x}", hasher.finalize()))
        }
        HashAlgo::Sha1 => {
            let mut hasher = Sha1::new();
            loop {
                let read = reader.read(&mut buffer).map_err(|e| format!("read_failed:{e}"))?;
                if read == 0 { break; }
                hasher.update(&buffer[..read]);
            }
            Ok(format!("{:x}", hasher.finalize()))
        }
        HashAlgo::Md5 => {
            let mut hasher = Md5::new();
            loop {
                let read = reader.read(&mut buffer).map_err(|e| format!("read_failed:{e}"))?;
                if read == 0 { break; }
                hasher.update(&buffer[..read]);
            }
            Ok(format!("{:x}", hasher.finalize()))
        }
        HashAlgo::Md4 => {
            let mut hasher = Md4::new();
            loop {
                let read = reader.read(&mut buffer).map_err(|e| format!("read_failed:{e}"))?;
                if read == 0 { break; }
                hasher.update(&buffer[..read]);
            }
            Ok(format!("{:x}", hasher.finalize()))
        }
    }
}
