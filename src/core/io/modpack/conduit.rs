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

pub struct ConduitProvider;

impl ModpackProvider for ConduitProvider {
    fn export(&self, output_path: &Path, include_config: bool) -> CoreResult<()> {
        let manifest_content = fs::read_to_string("conduit.json")?;
        let manifest: crate::core::io::project::manifest::ConduitConfig =
            serde_json::from_str(&manifest_content)?;

        let file = File::create(output_path)?;
        let mut zip = zip::ZipWriter::new(file);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        zip.start_file("conduit.json", options)?;
        zip.write_all(manifest_content.as_bytes())?;

        if Path::new("conduit.lock").exists() {
            zip.start_file("conduit.lock", options)?;
            zip.write_all(&fs::read("conduit.lock")?)?;
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
                has_configs: include_config && Path::new("config").exists(),
                has_mods_overrides: Path::new("mods").exists(),
            },
        };

        zip.start_file("metadata.toml", options)?;
        zip.write_all(meta.to_toml()?.as_bytes())?;

        let mut folders_to_include = Vec::new();
        if include_config {
            folders_to_include.push("config");
        }
        if Path::new("mods").exists() {
            folders_to_include.push("mods");
        }

        for folder in folders_to_include {
            for entry in WalkDir::new(folder).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() {
                    let zip_path =
                        format!("overrides/{}", path.to_string_lossy().replace('\\', "/"));
                    zip.start_file(zip_path, options)?;
                    zip.write_all(&fs::read(path)?)?;
                }
            }
        }

        zip.finish()
            .map_err(|e| CoreError::RuntimeError(e.to_string()))?;
        Ok(())
    }

    fn import(&self, input_path: &Path, target_dir: &Path) -> CoreResult<()> {
        let file = File::open(input_path)?;
        let mut archive =
            zip::ZipArchive::new(file).map_err(|e| CoreError::RuntimeError(e.to_string()))?;

        let mut suspicious_files = Vec::new();
        for i in 0..archive.len() {
            let file = archive
                .by_index(i)
                .map_err(|e| CoreError::RuntimeError(e.to_string()))?;
            if file.name().contains("overrides/") && file.name().ends_with(".jar") {
                suspicious_files.push(file.name().to_string());
            }
        }

        if !suspicious_files.is_empty() {
            println!("⚠️  WARNING: This modpack includes local .jar files in overrides.");
            println!(
                "Trust only packs from sources you know. Files: {:?}",
                suspicious_files
            );
        }

        if !target_dir.exists() {
            fs::create_dir_all(target_dir)?;
        }

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| CoreError::RuntimeError(e.to_string()))?;
            let raw_name = file.name().to_string();

            let outpath = if raw_name.starts_with("overrides/") {
                target_dir.join(raw_name.strip_prefix("overrides/").unwrap())
            } else {
                target_dir.join(raw_name)
            };

            if file.is_dir() {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    fs::create_dir_all(p)?;
                }
                let mut outfile = File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }

        Ok(())
    }
}
