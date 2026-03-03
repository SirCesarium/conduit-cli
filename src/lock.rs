use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
pub struct ConduitLock {
    pub version: i32,
    pub locked_mods: HashMap<String, LockedMod>,
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

impl ConduitLock {
    pub fn load() -> Self {
        fs::read_to_string("conduit-lock.toml")
            .ok()
            .and_then(|content| toml::from_str(&content).ok())
            .unwrap_or_else(|| ConduitLock {
                version: 1,
                locked_mods: HashMap::new(),
            })
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        fs::write("conduit-lock.toml", content)?;
        Ok(())
    }
}
