use serde::{Deserialize, Serialize};
use crate::core::error::CoreResult;
use crate::core::io::manifest::InstanceType;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConduitPackMetadata {
    pub version: String,
    pub pack_type: InstanceType,
    pub creator: String,
    pub has_configs: bool,
    pub has_mods_overrides: bool,
}

impl ConduitPackMetadata {
    pub fn to_toml(&self) -> CoreResult<String> {
        toml::to_string(self).map_err(|e| {
            crate::core::error::CoreError::RuntimeError(format!("Failed to serialize metadata: {}", e))
        })
    }

    pub fn from_toml(content: &str) -> CoreResult<Self> {
        toml::from_str(content).map_err(|e| {
            crate::core::error::CoreError::RuntimeError(format!("Failed to parse metadata: {}", e))
        })
    }
}