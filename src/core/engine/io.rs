use serde::{Serialize, de::DeserializeOwned};
use std::path::Path;
use tokio::fs;

use crate::{
    errors::ConduitResult,
    core::schemas::{lock, manifest},
};

pub trait TomlFile: Serialize + DeserializeOwned {
    async fn load<P: AsRef<Path> + Send>(path: P) -> ConduitResult<Self> {
        let content = fs::read_to_string(path).await?;
        let data = toml::from_str(&content)?;
        Ok(data)
    }

    async fn save<P: AsRef<Path> + Send>(&self, path: P) -> ConduitResult<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content).await?;
        Ok(())
    }
}

impl TomlFile for manifest::Manifest {}
impl TomlFile for lock::Lockfile {}
