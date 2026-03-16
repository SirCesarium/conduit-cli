use std::{
    fs::File,
    path::{Path, PathBuf},
};

use crate::{
    core::{
        engine::{archive::SafeArchive, io::IncludeFile},
        schemas::{include::ConduitInclude, lock::Lockfile, manifest::Manifest},
    },
    errors::{ConduitError, ConduitResult},
    paths::ConduitPaths,
};

pub struct ConduitModpackManager {
    pub manifest: Manifest,
    pub lock: Lockfile,
    pub include: ConduitInclude,
}

impl ConduitModpackManager {
    pub fn new(
        path: PathBuf,
        manifest: Manifest,
        lock: Lockfile,
        include: ConduitInclude,
        root: &Path,
    ) -> ConduitResult<Self> {
        let mut writer = SafeArchive::create(path)?;

        SafeArchive::serialize_and_add(&mut writer, ConduitPaths::manifest_name(), &manifest)?;
        SafeArchive::serialize_and_add(&mut writer, ConduitPaths::lockfile_name(), &lock)?;

        let include_content = include.paths.join("\n");
        SafeArchive::add_file(
            &mut writer,
            ConduitPaths::include_name(),
            include_content.as_bytes(),
        )?;

        for file_path in include.scan(root) {
            if file_path.is_file() {
                let relative = file_path
                    .strip_prefix(root)
                    .map_err(|_| ConduitError::Storage("Failed to strip prefix".to_string()))?;

                let zip_path = format!(
                    "overrides/{}",
                    relative.to_string_lossy().replace('\\', "/")
                );

                let file = File::open(&file_path)?;
                SafeArchive::add_file_from_reader(&mut writer, &zip_path, file)?;
            }
        }

        writer.finish().map_err(|e| {
            ConduitError::Storage(format!("Failed to finalize conduit modpack: {e}"))
        })?;

        Ok(Self {
            manifest,
            lock,
            include,
        })
    }

    pub fn open(path: PathBuf) -> ConduitResult<Self> {
        let mut zip = SafeArchive::open(path)?;

        let manifest = SafeArchive::read_and_deserialize(&mut zip, ConduitPaths::manifest_name())?;
        let lock = SafeArchive::read_and_deserialize(&mut zip, ConduitPaths::lockfile_name())?;

        let include_raw = SafeArchive::read_metadata(&mut zip, ConduitPaths::include_name())?;
        let patterns = include_raw
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();

        let include = ConduitInclude { paths: patterns };

        Ok(Self {
            manifest,
            lock,
            include,
        })
    }

    pub fn extract_to(&self, archive_path: PathBuf, destination: &Path) -> ConduitResult<()> {
        let mut zip = SafeArchive::open(archive_path)?;
        SafeArchive::extract_prefix(&mut zip, "overrides/", destination)
    }
}
