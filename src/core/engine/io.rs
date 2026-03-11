use serde::{Serialize, de::DeserializeOwned};
use std::path::Path;
use tokio::fs;

use crate::{
    core::schemas::{lock, manifest},
    errors::ConduitResult,
};

pub trait TomlFile: Serialize + DeserializeOwned + Sync {
    fn load<P: AsRef<Path> + Send>(
        path: P,
    ) -> impl std::future::Future<Output = ConduitResult<Self>> + Send {
        async {
            let content = fs::read_to_string(path).await?;
            let data = toml::from_str(&content)?;
            Ok(data)
        }
    }

    fn save<P: AsRef<Path> + Send>(
        &self,
        path: P,
    ) -> impl std::future::Future<Output = ConduitResult<()>> + Send {
        async move {
            let content = toml::to_string_pretty(self)?;
            fs::write(path, content).await?;
            Ok(())
        }
    }
}

impl TomlFile for manifest::Manifest {}
impl TomlFile for lock::Lockfile {}
