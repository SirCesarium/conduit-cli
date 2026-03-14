use std::{collections::HashMap, path::PathBuf};

use miette::IntoDiagnostic;

use crate::{
    core::{engine::archive::SafeArchive, schemas::modpacks::modrinth::ModrinthIndex},
    errors::{ConduitError, ConduitResult},
};

pub struct MrPackManager {
    file: ModrinthIndex,
}

impl MrPackManager {
    pub fn new(
        path: PathBuf,
        index: ModrinthIndex,
        extra_files: HashMap<String, Vec<u8>>,
    ) -> ConduitResult<Self> {
        let mut writer = SafeArchive::create(&path)?;

        let index_json = serde_json::to_vec(&index)
            .into_diagnostic()
            .map_err(|e| ConduitError::Parsing(e.to_string()))?;

        SafeArchive::add_file(&mut writer, "modrinth.index.json", &index_json)?;

        for (name, content) in extra_files {
            SafeArchive::add_file(&mut writer, &name, &content)?;
        }

        writer
            .finish()
            .map_err(|e| ConduitError::Storage(format!("Failed to finalize mrpack: {e}")))?;

        Ok(Self { file: index })
    }

    pub fn open(path: PathBuf) -> ConduitResult<Self> {
        let mut zip = SafeArchive::open(path)?;

        let file: ModrinthIndex =
            SafeArchive::read_and_deserialize(&mut zip, "modrinth.index.json")?;

        Ok(MrPackManager { file })
    }
}
