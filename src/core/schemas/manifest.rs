use crate::core::domain::loader::Loader;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Manifest {
    pub project: ProjectInfo,

    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub mods: BTreeMap<String, String>,

    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub plugins: BTreeMap<String, String>,

    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub datapacks: BTreeMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectInfo {
    pub name: String,
    pub minecraft: String,
    pub loader: Loader,
}

impl Default for ProjectInfo {
    fn default() -> Self {
        Self {
            name: "new-conduit-project".to_string(),
            minecraft: "1.21.11".to_string(),
            loader: Loader::Vanilla,
        }
    }
}
