use crate::core::error::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum InstanceType {
    Server,
    Client,
}

impl fmt::Display for InstanceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InstanceType::Server => write!(f, "server"),
            InstanceType::Client => write!(f, "client"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConduitConfig {
    pub name: String,

    #[serde(rename = "type")]
    pub instance_type: InstanceType,

    pub mc_version: String,
    pub loader: String,
    pub mods: BTreeMap<String, String>,
}

impl ConduitConfig {
    pub fn from_json(content: &str) -> CoreResult<Self> {
        serde_json::from_str(content)
            .map_err(|e| CoreError::RuntimeError(format!("Failed to parse conduit.json: {e}")))
    }

    pub fn to_json(&self) -> CoreResult<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| CoreError::RuntimeError(format!("Failed to serialize manifest: {e}")))
    }
}

impl Default for ConduitConfig {
    fn default() -> Self {
        Self {
            name: "conduit-project".to_string(),
            instance_type: InstanceType::Server,
            mc_version: "1.21.1".to_string(),
            loader: "neoforge@latest".to_string(),
            mods: BTreeMap::new(),
        }
    }
}
