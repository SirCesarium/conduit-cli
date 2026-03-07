use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;

use crate::core::error::{CoreError, CoreResult};
use crate::core::io::modpack::ModpackProvider;
use crate::core::io::modpack::metadata::{
    ConduitInfo, ConduitPackMetadata, ContentFlags, PackInfo,
};
use crate::core::io::project::ProjectFiles;
use crate::core::paths::CorePaths;

pub struct ConduitProvider;

impl ModpackProvider for ConduitProvider {
    fn export(&self, paths: &CorePaths, output_path: &Path, include_config: bool) -> CoreResult<()> {
        let manifest = ProjectFiles::load_manifest(paths)?;
        
        let file = File::create(output_path)?;
        let mut zip = zip::ZipWriter::new(file);
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

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
                has_mods_overrides: paths.mods_dir().exists(),
            },
        };

        zip.start_file("metadata.toml", options)?;
        zip.write_all(meta.to_toml()?.as_bytes())?;

        let mut folders = Vec::new();
        if include_config { folders.push("config"); }
        if paths.mods_dir().exists() { folders.push("mods"); }

        for folder_name in folders {
            let folder_path = paths.project_dir().join(folder_name);
            for entry in WalkDir::new(&folder_path).into_iter().filter_map(|e| e.ok()) {
                if entry.path().is_file() {
                    let relative_path = entry.path().strip_prefix(paths.project_dir())
                        .map_err(|_| CoreError::RuntimeError("Path error".into()))?;
                    
                    let zip_path = format!("overrides/{}", relative_path.to_string_lossy().replace('\\', "/"));
                    zip.start_file(zip_path, options)?;
                    zip.write_all(&fs::read(entry.path())?)?;
                }
            }
        }

        zip.finish().map_err(|e| CoreError::RuntimeError(e.to_string()))?;
        Ok(())
    }

    fn import(&self, paths: &CorePaths, input_path: &Path) -> CoreResult<()> {
        let file = File::open(input_path)?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| CoreError::RuntimeError(e.to_string()))?;

        for i in 0..archive.len() {
            let file = archive.by_index(i).map_err(|e| CoreError::RuntimeError(e.to_string()))?;
            if file.name().contains("overrides/") && file.name().ends_with(".jar") {
                println!("⚠️  WARNING: Modpack contains local .jar: {}", file.name());
            }
        }

        let target_dir = paths.project_dir();
        if !target_dir.exists() { fs::create_dir_all(target_dir)?; }

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| CoreError::RuntimeError(e.to_string()))?;
            let raw_name = file.name().to_string();

            let outpath = if let Some(stripped) = raw_name.strip_prefix("overrides/") {
                target_dir.join(stripped)
            } else {
                target_dir.join(&raw_name)
            };

            if file.is_dir() {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() { fs::create_dir_all(p)?; }
                let mut outfile = File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }

        Ok(())
    }
}