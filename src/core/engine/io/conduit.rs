use std::path::PathBuf;

use crate::{
    core::{
        engine::archive::SafeArchive,
        schemas::{lock::Lockfile, manifest::Manifest},
    },
    errors::{ConduitError, ConduitResult},
    paths::ConduitPaths,
};

pub struct ConduitModpackManager {
    manifest: Manifest,
    lock: Lockfile,
}

impl ConduitModpackManager {
    pub fn new(path: PathBuf, manifest: Manifest, lock: Lockfile) -> ConduitResult<Self> {
        let mut writer = SafeArchive::create(path)?;

        let manifest_path = ConduitPaths::manifest_name();
        let lockfile_path = ConduitPaths::lockfike_name();

        let manifest_content = toml::to_string(&manifest)?.into_bytes();
        let lockfile_content = toml::to_string(&lock)?.into_bytes();

        SafeArchive::add_file(&mut writer, manifest_path, &manifest_content)?;
        SafeArchive::add_file(&mut writer, lockfile_path, &lockfile_content)?;

        writer.finish().map_err(|e| {
            ConduitError::Storage(format!("Failed to finalize conduit modpack: {e}"))
        })?;

        Ok(Self { manifest, lock })
    }

    pub fn open(path: PathBuf) -> ConduitResult<Self> {
        let mut zip = SafeArchive::open(path)?;

        let manifest = SafeArchive::read_and_deserialize(&mut zip, ConduitPaths::manifest_name())?;
        let lock = SafeArchive::read_and_deserialize(&mut zip, ConduitPaths::lockfike_name())?;

        Ok(Self { manifest, lock })
    }
}
