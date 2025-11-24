use crate::state::AppState;
use blake3::Hasher as Blake3;
use crc32fast::Hasher as Crc32;
use md4::Md4;
use md5::Md5;
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
        }
        Err(e) => {
            state.last_error = Some(e);
            state.last_hash = None;
        }
    }
}

enum HashSource<'a> {
    RawFd(RawFd),
    Path(&'a str),
}

fn compute_hash(source: HashSource<'_>, algo: HashAlgo) -> Result<String, String> {
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
