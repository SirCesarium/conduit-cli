use crate::engine::io::TomlFile;
use crate::engine::workflow::Workflow;
use crate::errors::ConduitResult;
use crate::paths::ConduitPaths;
use crate::schemas::lock::{InstanceSnapshot, Lockfile};
use crate::schemas::manifest::Manifest;

impl Workflow {
    pub async fn migration(
        &self,
        manifest: &Manifest,
        lockfile: &Lockfile,
    ) -> ConduitResult<Lockfile> {
        let current_loader = &lockfile.instance.loader;
        let current_mc = &lockfile.instance.minecraft_version;
        let target_loader = &manifest.project.loader;
        let target_mc = &manifest.project.minecraft;

        let mut active_lock = lockfile.clone();

        if lockfile.instance.loader_hash.is_some()
            && (current_loader != target_loader || current_mc != target_mc)
        {
            let old_id = ConduitPaths::get_runtime_id(current_loader, current_mc);
            let runtime_path = self.project_root.join(".conduit_runtimes").join(old_id);

            tokio::fs::create_dir_all(&runtime_path).await?;

            let mut entries = tokio::fs::read_dir(&self.project_root).await?;
            while let Some(entry) = entries.next_entry().await? {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();

                if ConduitPaths::is_conduit_file(&name_str) {
                    continue;
                }

                let _ = tokio::fs::rename(entry.path(), runtime_path.join(&name)).await;
            }

            let manifest_path = ConduitPaths::get_manifest_path(&self.project_root);
            let lock_path = ConduitPaths::get_lock_path(&self.project_root);

            let mut old_instance_manifest =
                Manifest::load(&manifest_path).await.unwrap_or_default();
            old_instance_manifest.project.loader = current_loader.clone();
            old_instance_manifest.project.minecraft = current_mc.clone();

            old_instance_manifest
                .save(runtime_path.join("conduit.toml"))
                .await?;

            let _ = tokio::fs::copy(&lock_path, runtime_path.join("conduit.lock")).await;

            let mut clean_manifest = Manifest::load(&manifest_path).await.unwrap_or_default();
            clean_manifest.plugins.clear();
            clean_manifest.mods.clear();
            clean_manifest.datapacks.clear();
            clean_manifest.save(&manifest_path).await?;

            let clean_lock = Lockfile {
                instance: InstanceSnapshot {
                    minecraft_version: target_mc.clone(),
                    loader: target_loader.clone(),
                    loader_hash: None,
                    hash_kind: None,
                },
                ..Default::default()
            };

            clean_lock.save(&lock_path).await?;
            active_lock = clean_lock;

            let mut ctx_manifest = self.ctx.manifest.write().await;
            *ctx_manifest = clean_manifest;
        }

        let new_id = ConduitPaths::get_runtime_id(target_loader, target_mc);
        let target_runtime_path = self.project_root.join(".conduit_runtimes").join(&new_id);

        if target_runtime_path.exists() {
            let mut entries = tokio::fs::read_dir(&target_runtime_path).await?;
            while let Some(entry) = entries.next_entry().await? {
                let name = entry.file_name();
                let _ = tokio::fs::rename(entry.path(), self.project_root.join(&name)).await;
            }

            let manifest_path = ConduitPaths::get_manifest_path(&self.project_root);
            let lock_path = ConduitPaths::get_lock_path(&self.project_root);

            let restored_manifest = Manifest::load(&manifest_path).await.unwrap_or_default();
            let restored_lock = Lockfile::load(&lock_path).await.unwrap_or_default();

            let mut ctx_manifest = self.ctx.manifest.write().await;
            *ctx_manifest = restored_manifest;

            active_lock = restored_lock;

            let _ = tokio::fs::remove_dir_all(target_runtime_path).await;
        }

        Ok(active_lock)
    }
}
