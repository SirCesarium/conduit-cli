use std::{
    collections::HashMap,
    fs::File,
    path::{Path, PathBuf},
};

use crate::{
    core::{engine::archive::SafeArchive, schemas::modpacks::modrinth::ModrinthIndex},
    errors::{ConduitError, ConduitResult},
};

pub struct MrPackManager {
    pub file: ModrinthIndex,
}

impl MrPackManager {
    pub fn new(
        path: &PathBuf,
        index: ModrinthIndex,
        extra_files: HashMap<String, Vec<u8>>,
        overrides: Option<(PathBuf, Vec<PathBuf>)>,
    ) -> ConduitResult<Self> {
        let mut writer = SafeArchive::create(path)?;

        SafeArchive::serialize_and_add(&mut writer, "modrinth.index.json", &index)?;

        for (name, content) in extra_files {
            SafeArchive::add_file(&mut writer, &name, &content)?;
        }

        if let Some((root, files)) = overrides {
            for file_path in files {
                if file_path.is_file() {
                    let relative = file_path.strip_prefix(&root).map_err(|_| {
                        ConduitError::Storage(
                            "Failed to strip prefix for mrpack override".to_string(),
                        )
                    })?;

                    let zip_path = format!(
                        "overrides/{}",
                        relative.to_string_lossy().replace('\\', "/")
                    );

                    let file = File::open(&file_path)?;
                    SafeArchive::add_file_from_reader(&mut writer, &zip_path, file)?;
                }
            }
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

    pub fn extract_overrides(
        &self,
        archive_path: PathBuf,
        destination: &Path,
    ) -> ConduitResult<()> {
        let mut zip = SafeArchive::open(archive_path)?;
        SafeArchive::extract_prefix(&mut zip, "overrides/", destination)
    }
}
