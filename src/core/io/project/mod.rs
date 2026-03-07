pub mod manifest;
pub mod lock;

use std::fs;
use crate::core::error::{CoreError, CoreResult};
use crate::core::paths::CorePaths;
pub use manifest::{ConduitConfig, InstanceType};
pub use lock::ConduitLock;

pub struct ProjectFiles;

impl ProjectFiles {
    pub fn load_manifest(paths: &CorePaths) -> CoreResult<ConduitConfig> {
        let content = fs::read_to_string(paths.manifest_path())
            .map_err(|_| CoreError::MissingConfig)?;
        ConduitConfig::from_json(&content)
    }

    pub fn save_manifest(paths: &CorePaths, config: &ConduitConfig) -> CoreResult<()> {
        let json = config.to_json()?;
        fs::write(paths.manifest_path(), json)
            .map_err(|e| CoreError::RuntimeError(format!("Failed to write conduit.json: {}", e)))
    }

    pub fn load_lock(paths: &CorePaths) -> CoreResult<ConduitLock> {
        let path = paths.lock_path();
        if !path.exists() {
            return Ok(ConduitLock::default());
        }
        let content = fs::read_to_string(path)
            .map_err(|e| CoreError::RuntimeError(format!("Failed to read conduit.lock: {}", e)))?;
        ConduitLock::from_toml(&content)
    }

    pub fn save_lock(paths: &CorePaths, lock: &ConduitLock) -> CoreResult<()> {
        let toml = lock.to_toml_with_header()?;
        fs::write(paths.lock_path(), toml)
            .map_err(|e| CoreError::RuntimeError(format!("Failed to write conduit.lock: {}", e)))
    }
}
