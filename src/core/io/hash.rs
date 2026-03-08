use crate::core::error::CoreResult;
use sha2::Digest;
use std::fmt::Write;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

fn hash_file<D: Digest, P: AsRef<Path>>(path: P) -> CoreResult<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = D::new();
    let mut buf = [0u8; 8192];
    
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    let result = hasher.finalize();

    let mut hash_string = String::with_capacity(result.len() * 2);
    
    for b in result {
        let _ = write!(hash_string, "{b:02x}");
    }

    Ok(hash_string)
}

pub fn sha256_file<P: AsRef<Path>>(path: P) -> CoreResult<String> {
    hash_file::<sha2::Sha256, _>(path)
}

pub fn sha1_file<P: AsRef<Path>>(path: P) -> CoreResult<String> {
    hash_file::<sha1::Sha1, _>(path)
}
