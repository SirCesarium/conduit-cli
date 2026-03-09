use crate::core::{
    error::{CoreError, CoreResult},
};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct CorePaths {
    pub project_dir: PathBuf,
    pub manifest_path: PathBuf,
    pub lock_path: PathBuf,
    pub mods_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub config_path: PathBuf,
}

impl CorePaths {
    pub fn from_project_dir(project_dir: impl Into<PathBuf>) -> CoreResult<Self> {
        let project_dir = project_dir.into();
        let cache_dir = dirs::data_local_dir()
            .ok_or(CoreError::MissingLocalDataDir)?
            .join("conduit")
            .join("cache");

        if !cache_dir.exists() {
            std::fs::create_dir_all(&cache_dir)
                .map_err(|e| CoreError::RuntimeError(e.to_string()))?;
        }

        Ok(Self {
            manifest_path: project_dir.join("conduit.json"),
            lock_path: project_dir.join("conduit.lock"),
            config_path: project_dir.join("config.toml"),
            mods_dir: project_dir.join("mods"),
            project_dir,
            cache_dir,
        })
    }

    pub fn manifest_path(&self) -> &Path {
        &self.manifest_path
    }

    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    pub fn project_dir(&self) -> &Path {
        &self.project_dir
    }

    pub fn lock_path(&self) -> &Path {
        &self.lock_path
    }

    pub fn mods_dir(&self) -> &Path {
        &self.mods_dir
    }

    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    pub fn loader_libs_dir(&self) -> PathBuf {
        self.project_dir.join("libraries")
    }
}
