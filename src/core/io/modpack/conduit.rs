use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{Read, Write, copy};
use std::path::Path;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;

use crate::core::error::{CoreError, CoreResult};
use crate::core::events::{CoreCallbacks, CoreEvent};
use crate::core::io::modpack::metadata::{
    ConduitInfo, ConduitPackMetadata, ContentFlags, PackInfo,
};
use crate::core::io::modpack::{ModpackProvider, PackAnalysis};
use crate::core::io::project::ProjectFiles;
use crate::core::paths::CorePaths;

pub struct ConduitProvider;

impl ConduitProvider {
    const MAX_DECOMPRESSION_RATIO: u64 = 200;
    const MAX_TOTAL_SIZE: u64 = 1024 * 1024 * 1024 * 2;
    const MAX_SINGLE_FILE_SIZE: u64 = 1024 * 1024 * 500;

    fn is_dangerous(extension: &str) -> bool {
        let blacklist = [
            "exe", "bat", "sh", "py", "js", "vbs", "msi", "com", "cmd", "scr",
        ];
        blacklist.contains(&extension.to_lowercase().as_str())
    }

    pub fn validate_zip_entry<R>(
        file: &zip::read::ZipFile<'_, R>,
        current_total: u64,
    ) -> CoreResult<u64>
    where
        R: std::io::Read + std::io::Seek,
    {
        let name = file.name();
        let uncompressed_size = file.size();
        let compressed_size = file.compressed_size();

        if uncompressed_size > Self::MAX_SINGLE_FILE_SIZE {
            return Err(CoreError::RuntimeError(format!("File too large: {name}")));
        }

        if compressed_size > 0 {
            let ratio = uncompressed_size / compressed_size;
            if ratio > Self::MAX_DECOMPRESSION_RATIO && uncompressed_size > 1024 * 1024 {
                return Err(CoreError::RuntimeError(format!(
                    "Abnormal compression ratio: {name}"
                )));
            }
        }

        let new_total = current_total + uncompressed_size;
        if new_total > Self::MAX_TOTAL_SIZE {
            return Err(CoreError::RuntimeError(
                "Pack exceeds total size limit".into(),
            ));
        }

        Ok(new_total)
    }
}

impl ModpackProvider for ConduitProvider {
    fn export(
        &self,
        paths: &CorePaths,
        output_path: &Path,
        include_config: bool,
    ) -> CoreResult<()> {
        let manifest = ProjectFiles::load_manifest(paths)?;
        let mut local_mod_filenames = HashSet::new();

        if paths.lock_path().exists() {
            let lock = ProjectFiles::load_lock(paths)?;
            for (slug, locked_mod) in lock.locked_mods {
                if slug.starts_with("local:") || slug.starts_with("f:") || slug.starts_with("file:")
                {
                    local_mod_filenames.insert(locked_mod.filename);
                }
            }
        }

        let file = File::create(output_path)?;
        let mut zip = zip::ZipWriter::new(file);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        zip.start_file("conduit.json", options)?;
        zip.write_all(manifest.to_json()?.as_bytes())?;

        if paths.lock_path().exists() {
            let lock = ProjectFiles::load_lock(paths)?;
            zip.start_file("conduit.lock", options)?;
            zip.write_all(lock.to_toml_with_header()?.as_bytes())?;
        }

        let meta = ConduitPackMetadata {
            conduit: ConduitInfo {
                version: env!("CARGO_PKG_VERSION").to_string(),
                format_version: 1,
            },
            pack: PackInfo {
                title: manifest.name.clone(),
                creator: Some("Conduit User".to_string()),
                description: None,
                homepage: None,
                repository: None,
                pack_type: manifest.instance_type,
            },
            content: ContentFlags {
                has_configs: include_config && paths.project_dir().join("config").exists(),
                has_mods_overrides: !local_mod_filenames.is_empty(),
            },
        };

        zip.start_file("metadata.toml", options)?;
        zip.write_all(meta.to_toml()?.as_bytes())?;

        let mut folders = Vec::new();
        if include_config {
            folders.push("config");
        }
        if paths.mods_dir().exists() {
            folders.push("mods");
        }

        for folder_name in folders {
            let folder_path = paths.project_dir().join(folder_name);
            let is_mods_folder = folder_name == "mods";

            for entry in WalkDir::new(&folder_path)
                .into_iter()
                .filter_map(Result::ok)
            {
                if entry.path().is_file() {
                    let filename = entry.file_name().to_string_lossy().to_string();

                    if is_mods_folder && !local_mod_filenames.contains(&filename) {
                        continue;
                    }

                    let relative_path = entry
                        .path()
                        .strip_prefix(paths.project_dir())
                        .map_err(|_| CoreError::RuntimeError("Path error".into()))?;

                    let zip_path = format!(
                        "overrides/{}",
                        relative_path.to_string_lossy().replace('\\', "/")
                    );

                    zip.start_file(zip_path, options)?;
                    zip.write_all(&fs::read(entry.path())?)?;
                }
            }
        }

        zip.finish()
            .map_err(|e| CoreError::RuntimeError(e.to_string()))?;
        Ok(())
    }

    fn analyze(&self, input_path: &Path) -> CoreResult<PackAnalysis> {
        let file = File::open(input_path)?;
        let mut archive =
            zip::ZipArchive::new(file).map_err(|e| CoreError::RuntimeError(e.to_string()))?;

        let mut files = Vec::new();
        let mut extensions = HashSet::new();
        let mut suspicious = Vec::new();
        let mut dangerous_count = 0;
        let mut local_jars_count = 0;
        let mut total_uncompressed_size: u64 = 0;

        for i in 0..archive.len() {
            let file = archive
                .by_index(i)
                .map_err(|e| CoreError::RuntimeError(e.to_string()))?;
            let name = file.name().to_string();

            if file.is_file() {
                total_uncompressed_size =
                    ConduitProvider::validate_zip_entry(&file, total_uncompressed_size)?;

                files.push(name.clone());

                if let Some(ext) = Path::new(&name).extension().and_then(|e| e.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    extensions.insert(ext_lower.clone());

                    if Self::is_dangerous(&ext_lower) {
                        dangerous_count += 1;
                        suspicious.push(format!("[DANGER] {name}"));
                    }
                }

                if name.contains("overrides/")
                    && Path::new(&name)
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("jar"))
                {
                    local_jars_count += 1;
                    suspicious.push(format!("[LOCAL JAR] {name}"));
                }
            }
        }

        Ok(PackAnalysis {
            files,
            extensions: extensions.into_iter().collect(),
            dangerous_count,
            local_jars_count,
            suspicious_files: suspicious,
        })
    }

    fn import(
        &self,
        paths: &CorePaths,
        input_path: &Path,
        callbacks: &mut dyn CoreCallbacks,
    ) -> CoreResult<()> {
        let file = File::open(input_path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        let target_dir = paths.project_dir();
        if !target_dir.exists() {
            fs::create_dir_all(target_dir)?;
        }

        let mut total_uncompressed_size: u64 = 0;

        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            let raw_name = file.name().to_string();

            total_uncompressed_size =
                ConduitProvider::validate_zip_entry(&file, total_uncompressed_size)?;

            let outpath = if let Some(stripped) = raw_name.strip_prefix("overrides/") {
                target_dir.join(stripped)
            } else {
                target_dir.join(&raw_name)
            };

            if !outpath.starts_with(target_dir) {
                return Err(CoreError::RuntimeError(format!(
                    "Invalid path in ZIP: {raw_name}"
                )));
            }

            if file.is_dir() {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    fs::create_dir_all(p)?;
                }
                let mut outfile = File::create(&outpath)?;
                let mut limiter = file.take(Self::MAX_SINGLE_FILE_SIZE);
                copy(&mut limiter, &mut outfile)?;

                if Path::new(&raw_name)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
                    || Path::new(&raw_name)
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("lock"))
                {
                    callbacks.on_event(CoreEvent::LinkedFile { filename: raw_name });
                }
            }
        }

        callbacks.on_event(CoreEvent::Success(
            "Modpack imported successfully".to_string(),
        ));
        Ok(())
    }
}
