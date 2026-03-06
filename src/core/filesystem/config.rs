use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConduitConfig {
    pub name: String,
    pub mc_version: String,
    pub loader: String,
    pub mods: BTreeMap<String, String>,
}
