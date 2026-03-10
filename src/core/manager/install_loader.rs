use std::io::{self, Error};
use std::process::Command;
use thiserror::Error;

use crate::api::ApiError;
use crate::core::downloader::DownloadError;
use crate::core::io::TomlFile;
use crate::core::manager::ProjectManager;
use crate::core::store::StoreError;
use crate::domain::loader::Loader;
use crate::domain::source::Hash;
use crate::paths::ConduitPaths;
use crate::schemas::lock::{InstanceSnapshot, Lockfile};
use crate::schemas::manifest::Manifest;

#[derive(Error, Debug)]
pub enum InstallError {
    #[error("api error: {0}")]
    Api(#[from] ApiError),
    #[error("download error: {0}")]
    Download(#[from] DownloadError),
    #[error("store error: {0}")]
    Store(#[from] StoreError),
    #[error("io error: {0}")]
    Io(#[from] io::Error),
}

impl ProjectManager {
    pub async fn install_loader(&self) -> Result<(), InstallError> {
        let manifest_path = ConduitPaths::get_manifest_path(&self.project_root);
        let lock_path = ConduitPaths::get_lock_path(&self.project_root);

        let manifest = Manifest::load(manifest_path)
            .await
            .map_err(|e| Error::new(io::ErrorKind::NotFound, e.to_string()))?;

        let mut lockfile = Lockfile::load(&lock_path).await.unwrap_or_default();

        if let (Some(h), Some(k)) = (&lockfile.instance.loader_hash, &lockfile.instance.hash_kind)
            && !h.is_empty()
            && lockfile.instance.loader == manifest.project.loader
            && lockfile.instance.minecraft_version == manifest.project.minecraft
            && let Ok((final_hash, kind)) = self.ctx.downloader.download_to_store_by_hash(h, *k)
        {
            let target_path = self.project_root.join("server.jar");
            self.ctx
                .store
                .link_object(&final_hash, kind, target_path)
                .await?;
            return Ok(());
        }

        let loader_info = &manifest.project.loader;
        let resolved = self.resolver.resolve_loader(loader_info).await?;

        let mut hash_obj = Hash {
            sha1: None,
            sha256: None,
            sha512: None,
        };
        if !resolved.hash.is_empty() {
            if resolved.hash.len() == 64 {
                hash_obj.sha256 = Some(resolved.hash.clone());
            } else {
                hash_obj.sha1 = Some(resolved.hash.clone());
            }
        }

        let (final_hash, kind) = self
            .ctx
            .downloader
            .download_to_store(
                &resolved.url,
                if resolved.hash.is_empty() {
                    None
                } else {
                    Some(&hash_obj)
                },
            )
            .await?;

        let is_installer = resolved.file_name.contains("installer");
        let target_name = if is_installer {
            "installer.jar"
        } else {
            "server.jar"
        };
        let target_path = self.project_root.join(target_name);

        if is_installer {
            let source = self.ctx.store.object_path(&final_hash, kind);
            tokio::fs::copy(source, &target_path).await?;
            self.run_installer(&target_path, loader_info)?;
            let _ = tokio::fs::remove_file(target_path).await;
        } else {
            self.ctx
                .store
                .link_object(&final_hash, kind, &target_path)
                .await?;
        }

        lockfile.instance = InstanceSnapshot {
            minecraft_version: manifest.project.minecraft.clone(),
            loader: manifest.project.loader.clone(),
            loader_hash: Some(final_hash),
            hash_kind: Some(kind),
        };

        lockfile
            .save(&lock_path)
            .await
            .map_err(|e| Error::other(e.to_string()))?;

        Ok(())
    }

    fn run_installer(&self, path: &std::path::Path, loader: &Loader) -> Result<(), InstallError> {
        let mut cmd = Command::new("java");
        cmd.arg("-jar").arg(path);

        match loader {
            Loader::Fabric { version } => {
                cmd.arg("server")
                    .arg("-mcversion")
                    .arg(version)
                    .arg("-downloadMinecraft");
            }
            _ => {
                cmd.arg("--installServer");
            }
        }

        let status = cmd
            .current_dir(&self.project_root)
            .status()
            .map_err(|e| io::Error::other(format!("failed to run installer: {e}")))?;

        if !status.success() {
            return Err(InstallError::Io(io::Error::other(
                "installer exited with error",
            )));
        }
        Ok(())
    }
}
