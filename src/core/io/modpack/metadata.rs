use serde::{Deserialize, Serialize};
use crate::core::error::{CoreError, CoreResult};
use crate::core::io::project::manifest::InstanceType;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConduitPackMetadata {
    pub conduit: ConduitInfo,
    pub pack: PackInfo,
    pub content: ContentFlags,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConduitInfo {
    pub version: String,
    pub format_version: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PackInfo {
    pub title: String,
    pub creator: Option<String>,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    #[serde(rename = "type")]
    pub pack_type: InstanceType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContentFlags {
    pub has_configs: bool,
    pub has_mods_overrides: bool,
}

impl ConduitPackMetadata {
    pub fn to_toml(&self) -> CoreResult<String> {
        toml::to_string(self).map_err(|e| {
            CoreError::RuntimeError(format!("Failed to serialize pack metadata: {e}"))
        })
    }

    pub fn from_toml(content: &str) -> CoreResult<Self> {
        toml::from_str(content).map_err(|e| {
            CoreError::RuntimeError(format!("Failed to parse pack metadata: {e}"))
        })
    }
}
