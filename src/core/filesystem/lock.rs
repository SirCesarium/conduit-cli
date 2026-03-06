use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ConduitLock {
    pub version: i32,
    pub locked_mods: HashMap<String, LockedMod>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loader_version: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LockedMod {
    pub id: String,
    pub version_id: String,
    pub filename: String,
    pub url: String,
    pub hash: String,
    pub dependencies: Vec<String>,
}
