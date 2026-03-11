use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs;
use tokio::io::AsyncReadExt;

use crate::errors::ConduitResult;
use crate::schemas::lock::HashKind;

#[derive(Clone, Debug)]
pub struct Store {
    root: PathBuf,
}

impl Store {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn object_path(&self, hash: &str, kind: HashKind) -> PathBuf {
        let prefix = match kind {
            HashKind::Sha1 => "sha1",
            HashKind::Sha256 => "sha256",
            HashKind::Sha512 => "sha512",
        };
        self.root
            .join("objects")
            .join(prefix)
            .join(&hash[..2])
            .join(hash)
    }

    pub async fn calculate_hash<P: AsRef<Path>>(
        &self,
        path: P,
        kind: HashKind,
    ) -> ConduitResult<String> {
        let mut file = fs::File::open(path).await?;
        let mut buffer = [0; 8192];

        match kind {
            HashKind::Sha1 => {
                let mut hasher = Sha1::new();
                while let Ok(n) = file.read(&mut buffer).await {
                    if n == 0 {
                        break;
                    }
                    hasher.update(&buffer[..n]);
                }
                Ok(format!("{:x}", hasher.finalize()))
            }
            HashKind::Sha256 => {
                let mut hasher = Sha256::new();
                while let Ok(n) = file.read(&mut buffer).await {
                    if n == 0 {
                        break;
                    }
                    hasher.update(&buffer[..n]);
                }
                Ok(format!("{:x}", hasher.finalize()))
            }
            HashKind::Sha512 => {
                let mut hasher = Sha512::new();
                while let Ok(n) = file.read(&mut buffer).await {
                    if n == 0 {
                        break;
                    }
                    hasher.update(&buffer[..n]);
                }
                Ok(format!("{:x}", hasher.finalize()))
            }
        }
    }

    pub async fn add_file<P: AsRef<Path>>(
        &self,
        source: P,
        hash: &str,
        kind: HashKind,
    ) -> ConduitResult<()> {
        let target = self.object_path(hash, kind);

        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).await?;
        }

        if !target.exists() {
            fs::copy(source, target).await?;
        }

        Ok(())
    }

    pub async fn link_object<P: AsRef<Path>>(
        &self,
        hash: &str,
        kind: HashKind,
        target: P,
    ) -> ConduitResult<()> {
        let source = self.object_path(hash, kind);

        if let Some(parent) = target.as_ref().parent() {
            fs::create_dir_all(parent).await?;
        }

        if target.as_ref().exists() {
            fs::remove_file(&target).await?;
        }

        fs::hard_link(source, target).await?;
        Ok(())
    }

    pub fn get_project_root(&self) -> PathBuf {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    }

    pub fn get_mods_path(&self) -> PathBuf {
        self.get_project_root().join("mods")
    }

    pub fn get_plugins_path(&self) -> PathBuf {
        self.get_project_root().join("plugins")
    }

    pub fn get_world_path(&self) -> PathBuf {
        self.get_project_root().join("world")
    }

    pub async fn install_to_project(
        &self,
        hash: &str,
        kind: HashKind,
        rel_path: PathBuf,
    ) -> ConduitResult<()> {
        let target = self.get_project_root().join(rel_path);
        self.link_object(hash, kind, target).await
    }
}
