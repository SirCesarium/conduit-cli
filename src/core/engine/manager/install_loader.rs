use crate::{
    engine::{io::TomlFile, manager::ProjectManager},
    errors::ConduitResult,
    paths::ConduitPaths,
    schemas::lock::InstanceSnapshot,
};

impl ProjectManager {
    pub async fn install_loader(&self) -> ConduitResult<()> {
        let manifest = self.ctx.manifest.read().await.clone();
        let lock = self.ctx.lockfile.read().await.clone();

        let mut active_lock = self.workflow.migration(&manifest, &lock).await?;

        if self
            .workflow
            .ensure_loader_presence(&active_lock, &manifest)?
        {
            let mut ctx_lock = self.ctx.lockfile.write().await;
            *ctx_lock = active_lock;
            return Ok(());
        }

        let loader_info = &manifest.project.loader;
        let resolved = self
            .resolver
            .resolve_loader(loader_info, &manifest.project.minecraft)
            .await?;

        let (final_hash, kind) = self.workflow.download_loader(&resolved).await?;

        self.workflow
            .execute_installation(
                &resolved,
                &final_hash,
                kind,
                loader_info,
                &manifest.project.minecraft,
            )
            .await?;

        active_lock.instance = InstanceSnapshot {
            minecraft_version: manifest.project.minecraft.clone(),
            loader: manifest.project.loader.clone(),
            loader_hash: Some(final_hash),
            hash_kind: Some(kind),
        };

        active_lock
            .save(ConduitPaths::get_lock_path(&self.project_root))
            .await?;

        let mut ctx_lock = self.ctx.lockfile.write().await;
        *ctx_lock = active_lock;

        Ok(())
    }
}
