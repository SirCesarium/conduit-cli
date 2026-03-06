use std::{collections::BTreeMap, fmt};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum InstanceType {
    Server,
    Client
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
