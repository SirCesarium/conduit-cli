use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum InstanceType {
    Server,
    Client
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
