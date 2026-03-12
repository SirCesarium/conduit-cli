use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncReadExt;

use crate::core::schemas::lock::HashKind;
use crate::errors::ConduitResult;

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
        let mut buffer = vec![0u8; 65536].into_boxed_slice();

        match kind {
            HashKind::Sha1 => {
                let mut hasher = Sha1::new();
                loop {
                    let n = file.read(&mut buffer).await?;
                    if n == 0 {
                        break;
                    }
                    hasher.update(&buffer[..n]);
                }
                Ok(format!("{:x}", hasher.finalize()))
            }
            HashKind::Sha256 => {
                let mut hasher = Sha256::new();
                loop {
                    let n = file.read(&mut buffer).await?;
                    if n == 0 {
                        break;
                    }
                    hasher.update(&buffer[..n]);
                }
                Ok(format!("{:x}", hasher.finalize()))
            }
            HashKind::Sha512 => {
                let mut hasher = Sha512::new();
                loop {
                    let n = file.read(&mut buffer).await?;
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

        if target.exists() {
            return Ok(());
        }

        if let Some(parent) = target.parent() {
            let _ = fs::create_dir_all(parent).await;
        }

        let temp_target = target.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));

        fs::copy(&source, &temp_target).await?;

        if let Err(e) = fs::rename(&temp_target, &target).await {
            let _ = fs::remove_file(&temp_target).await;
            if !target.exists() {
                return Err(e.into());
            }
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
