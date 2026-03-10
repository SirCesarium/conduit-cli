use sha2::{Digest, Sha512};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs;
use tokio::io::AsyncReadExt;

#[derive(Error, Debug)]
pub enum StoreError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("integrity check failed: expected {expected}, found {found}")]
    HashMismatch { expected: String, found: String },
}

#[derive(Clone, Debug)]
pub struct Store {
    root: PathBuf,
}

impl Store {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn object_path(&self, hash: &str) -> PathBuf {
        self.root.join("objects").join(&hash[..2]).join(hash)
    }

    pub async fn calculate_hash<P: AsRef<Path>>(&self, path: P) -> Result<String, StoreError> {
        let mut file = fs::File::open(path).await?;
        let mut hasher = Sha512::new();
        let mut buffer = [0; 8192];

        loop {
            let n = file.read(&mut buffer).await?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    pub async fn add_file<P: AsRef<Path>>(&self, source: P, hash: &str) -> Result<(), StoreError> {
        let target = self.object_path(hash);

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
        target: P,
    ) -> Result<(), StoreError> {
        let source = self.object_path(hash);

        if let Some(parent) = target.as_ref().parent() {
            fs::create_dir_all(parent).await?;
        }

        if target.as_ref().exists() {
            fs::remove_file(&target).await?;
        }

        fs::hard_link(source, target).await?;
        Ok(())
    }
}
