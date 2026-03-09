use crate::core::error::{CoreError, CoreResult};
use crate::core::io::hash::sha256_file;
use crate::core::io::project::lock::{LockedMod, ModSide};
use crate::core::io::project::{ConduitConfig, ConduitLock, ProjectFiles};
use crate::core::mods::inspector::JarInspector;
use crate::core::paths::CorePaths;
use std::collections::BTreeSet;
use std::fs;
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
    explicit_deps: Vec<PathBuf>,
    explicit_side: Option<&ModSide>,
) -> CoreResult<AddLocalModsReport> {
    let mut config = ProjectFiles::load_manifest(paths)?;
    let mut lock = ProjectFiles::load_lock(paths)?;

    fs::create_dir_all(paths.mods_dir())?;
    let mut report = AddLocalModsReport::default();

    let mut dep_keys = Vec::new();
    for dep_path in explicit_deps {
        let (key, _) = process_local_jar(
            paths,
            &dep_path,
            &mut config,
            &mut lock,
            explicit_side,
            Vec::new(),
            false
        )?;
        dep_keys.push(key);
    }

    for jar in jar_paths {
        let (key, filename) = process_local_jar(
            paths,
            &jar,
            &mut config,
            &mut lock,
            explicit_side,
            dep_keys.clone(),
            true
        )?;
        report.added.push(AddedLocalMod { key, filename });
    }

    ProjectFiles::save_manifest(paths, &config)?;
    ProjectFiles::save_lock(paths, &lock)?;
    Ok(report)
}

fn process_local_jar(
    paths: &CorePaths,
    jar_path: &Path,
    config: &mut ConduitConfig,
    lock: &mut ConduitLock,
    explicit_side: Option<&ModSide>,
    dependencies: Vec<String>,
    is_root: bool,
) -> CoreResult<(String, String)> {
    let jar = normalize_path(&paths.project_dir, jar_path);
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
    let side = explicit_side.copied().unwrap_or_else(|| JarInspector::detect_side(&jar));

    let id = key.clone();

    if is_root {
        config.mods.insert(key.clone(), "local".to_string());
    }
    
    lock.locked_mods.insert(
        key.clone(),
        LockedMod {
            id,
            version_id: "local".to_string(),
            filename: filename.clone(),
            url: "local".to_string(),
            hash: sha256,
            dependencies,
            side,
        },
    );

    Ok((key, filename))
}

#[derive(Debug, Clone, Default)]
pub struct MissingLocalReport {
    pub missing_files: Vec<String>,
    pub missing_lock_entries: Vec<String>,
}

pub fn find_missing_local_mods(paths: &CorePaths) -> CoreResult<MissingLocalReport> {
    let config = ProjectFiles::load_manifest(paths)?;
    let lock = ProjectFiles::load_lock(paths)?;

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
