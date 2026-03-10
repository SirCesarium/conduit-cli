use crate::domain::addon::Addon;
use crate::domain::loader::Loader;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Manifest {
    pub project: ProjectInfo,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub addons: Vec<Addon>,
}

#[derive(Debug, Serialize, Deserialize)]
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
