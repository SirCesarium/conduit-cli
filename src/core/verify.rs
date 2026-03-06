use crate::core::error::CoreResult;
use crate::core::io::ConduitLock;
use crate::core::paths::CorePaths;
use sha1::{Digest as Sha1Digest, Sha1};
use sha2::Sha256;
use std::fs;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub enum VerifyScope {
    Modrinth,
    Local,
}

#[derive(Debug, Clone, Default)]
pub struct VerifyReport {
    pub ok: usize,
    pub mismatch: Vec<VerifyMismatch>,
    pub missing: Vec<VerifyMissing>,
}

#[derive(Debug, Clone)]
pub struct VerifyMismatch {
    pub key: String,
    pub filename: String,
    pub expected: String,
    pub actual: String,
}

#[derive(Debug, Clone)]
pub struct VerifyMissing {
    pub key: String,
    pub filename: String,
}

pub fn verify_project(paths: &CorePaths, scope: VerifyScope) -> CoreResult<VerifyReport> {
    let lock = ConduitLock::load_lock(paths)?;

    let mut report = VerifyReport::default();

    for (key, m) in &lock.locked_mods {
        let is_local = m.url == "local";
        match scope {
            VerifyScope::Modrinth if is_local => continue,
            VerifyScope::Local if !is_local => continue,
            _ => {}
        }

        let mod_path = paths.mods_dir().join(&m.filename);
        if !mod_path.exists() {
            report.missing.push(VerifyMissing {
                key: key.clone(),
                filename: m.filename.clone(),
            });
            continue;
        }

        let actual = if is_local {
            sha256_file(&mod_path)?
        } else {
            sha1_file(&mod_path)?
        };

        if actual.eq_ignore_ascii_case(&m.hash) {
            report.ok += 1;
        } else {
            report.mismatch.push(VerifyMismatch {
                key: key.clone(),
                filename: m.filename.clone(),
                expected: m.hash.clone(),
                actual,
            });
        }
    }

    Ok(report)
}

fn sha1_file(path: &Path) -> CoreResult<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha1::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn sha256_file(path: &Path) -> CoreResult<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
