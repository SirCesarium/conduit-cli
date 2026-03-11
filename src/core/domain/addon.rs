use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::domain::loader::Loader;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AddonType {
    Mod,
    Plugin,
    Datapack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Addon {
    pub id: Uuid,
    pub slug: String,
    pub file_name: String,
    pub r#type: AddonType,
    pub loaders: Vec<Loader>,
    pub dependencies: Vec<Uuid>,
}
