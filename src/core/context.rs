use std::path::PathBuf;

use crate::core::apis::ConduitApis;
use crate::core::error::CoreResult;
use crate::core::io::project::{ConduitConfig, ConduitLock, ProjectFiles};
use crate::core::io::server::config::ServerConfig;
use crate::core::paths::CorePaths;

pub struct ConduitContext {
    pub paths: CorePaths,
    pub api: ConduitApis,
    pub manifest: ConduitConfig,
    pub lock: ConduitLock,
    pub config: ServerConfig,
}

impl ConduitContext {
    pub fn load(project_dir: impl Into<PathBuf>) -> CoreResult<Self> {
        let paths = CorePaths::from_project_dir(project_dir)?;

        let manifest = ProjectFiles::load_manifest(&paths)?;
        let lock = ProjectFiles::load_lock(&paths)?;
        let config = ServerConfig::load_or_create(paths.config_path())?;
        let api = ConduitApis::new();

        Ok(Self {
            paths,
            api,
            manifest,
            lock,
            config,
        })
    }

    pub fn save_state(&self) -> CoreResult<()> {
        ProjectFiles::save_manifest(&self.paths, &self.manifest)?;
        ProjectFiles::save_lock(&self.paths, &self.lock)?;
        Ok(())
    }
}
