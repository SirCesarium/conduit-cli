use crate::config::ConduitConfig;
use crate::core::error::{CoreError, CoreResult};
use crate::core::io::{load_config, load_lock, save_config, save_lock};
use crate::core::paths::CorePaths;
use crate::inspector::JarInspector;
use crate::lock::{ConduitLock, LockedMod};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct AddedLocalMod {
    pub key: String,
    pub filename: String,
}

#[derive(Debug, Clone, Default)]
pub struct AddLocalModsReport {
    pub added: Vec<AddedLocalMod>,
}

pub fn add_local_mods_to_project(
    paths: &CorePaths,
    jar_paths: Vec<PathBuf>,
) -> CoreResult<AddLocalModsReport> {
    let mut config: ConduitConfig = load_config(paths)?;
    let mut lock: ConduitLock = load_lock(paths)?;

    fs::create_dir_all(paths.mods_dir())?;

    let mut report = AddLocalModsReport::default();

    for jar in jar_paths {
        let jar = normalize_path(&paths.project_dir, &jar);
        if !jar.exists() {
            return Err(CoreError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File not found: {}", jar.display()),
            )));
        }

        let filename = jar
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| {
                CoreError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Invalid file name",
                ))
            })?
            .to_string();

        let dest_path = paths.mods_dir().join(&filename);
        if !dest_path.exists() {
            fs::copy(&jar, &dest_path)?;
        }

        let sha256 = sha256_file(&jar)?;
        let mod_id = JarInspector::extract_primary_mod_id(&jar).ok().flatten();
        let key = local_key(&filename, mod_id.as_deref());

        let id = format!("local:{}", &sha256[..std::cmp::min(8, sha256.len())]);

        config.mods.insert(key.clone(), "local".to_string());
        lock.locked_mods.insert(
            key.clone(),
            LockedMod {
                id,
                version_id: "local".to_string(),
                filename: filename.clone(),
                url: "local".to_string(),
                hash: sha256,
                dependencies: Vec::new(),
            },
        );

        report.added.push(AddedLocalMod { key, filename });
    }

    save_config(paths, &config)?;
    save_lock(paths, &lock)?;

    Ok(report)
}

#[derive(Debug, Clone, Default)]
pub struct MissingLocalReport {
    pub missing_files: Vec<String>,
    pub missing_lock_entries: Vec<String>,
}

pub fn find_missing_local_mods(paths: &CorePaths) -> CoreResult<MissingLocalReport> {
    let config = load_config(paths)?;
    let lock = load_lock(paths)?;

    let mut missing_files: BTreeSet<String> = BTreeSet::new();
    let mut missing_lock_entries: BTreeSet<String> = BTreeSet::new();

    for (key, value) in &config.mods {
        if value != "local" {
            continue;
        }

        if let Some(locked) = lock.locked_mods.get(key) {
            if locked.url != "local" {
                continue;
            }
            let on_disk = paths.mods_dir().join(&locked.filename).exists();
            if !on_disk {
                missing_files.insert(locked.filename.clone());
            }
        } else {
            missing_lock_entries.insert(key.clone());
        }
    }

    Ok(MissingLocalReport {
        missing_files: missing_files.into_iter().collect(),
        missing_lock_entries: missing_lock_entries.into_iter().collect(),
    })
}

fn normalize_path(project_dir: &Path, p: &Path) -> PathBuf {
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        project_dir.join(p)
    }
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

fn local_key(filename: &str, mod_id: Option<&str>) -> String {
    if let Some(id) = mod_id {
        return format!("local:{}", id.to_lowercase());
    }
    format!("local:{}", local_key_from_filename(filename))
}

fn local_key_from_filename(filename: &str) -> String {
    let stem = filename.strip_suffix(".jar").unwrap_or(filename);
    let mut out = String::new();
    let mut last_dash = false;
    for ch in stem.chars() {
        let c = ch.to_ascii_lowercase();
        let is_ok = c.is_ascii_alphanumeric();
        if is_ok {
            out.push(c);
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        "my-local-mod".to_string()
    } else {
        trimmed.to_string()
    }
}
