use serde::{Serialize, de::DeserializeOwned};
use std::path::Path;
use tokio::fs;
use thiserror::Error;

use crate::schemas::{lock, manifest};

#[derive(Error, Debug)]
pub enum PersistenceError {
    #[error("disk read/write error: {0}")]
    Io(#[from] std::io::Error),

    #[error("failed to parse TOML file: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("failed to serialize data to TOML: {0}")]
    Serialize(#[from] toml::ser::Error),
}

pub trait TomlFile: Serialize + DeserializeOwned {
    async fn load<P: AsRef<Path> + Send>(path: P) -> Result<Self, PersistenceError> {
        let content = fs::read_to_string(path).await?;
        let data = toml::from_str(&content)?;
        Ok(data)
    }

    async fn save<P: AsRef<Path> + Send>(&self, path: P) -> Result<(), PersistenceError> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content).await?;
        Ok(())
    }
}

impl TomlFile for manifest::Manifest {}
impl TomlFile for lock::Lockfile {}
