use crate::{
    core::{
        domain::loader::Loader,
        engine::{io::TomlFile, manager::ProjectManager},
        schemas::lock::InstanceSnapshot,
    },
    errors::ConduitResult,
};

impl ProjectManager {
    pub async fn install_loader(&self, loader: Loader) -> ConduitResult<()> {
        let mut manifest = self.ctx.manifest.read().await.clone();
        let lock = self.ctx.lockfile.read().await.clone();

        let resolved = self
            .resolver
            .resolve_loader(&loader, &manifest.project.minecraft)
            .await?;

        if let Loader::Forge { version } | Loader::Neoforge { version } = &loader
            && !version.contains(&manifest.project.minecraft) {
                return Err(crate::errors::ConduitError::Validation(format!(
                    "Loader version '{}' is not compatible with Minecraft {}",
                    version, manifest.project.minecraft
                )));
            }

        let mut active_lock = self.workflow.migration(&manifest, &lock).await?;

        if self
            .workflow
            .ensure_loader_presence(&active_lock, &manifest)?
        {
            let mut ctx_lock = self.ctx.lockfile.write().await;
            *ctx_lock = active_lock;
            return Ok(());
        }

        let (final_hash, kind) = self.workflow.download_loader(&resolved).await?;

        self.workflow
            .execute_installation(
                &resolved,
                &final_hash,
                kind,
                &loader,
                &manifest.project.minecraft,
            )
            .await?;

        active_lock.instance = InstanceSnapshot {
            minecraft_version: manifest.project.minecraft.clone(),
            loader: loader.clone(),
            loader_hash: Some(final_hash),
            hash_kind: Some(kind),
        };

        manifest.project.loader = loader;

        manifest.save(self.ctx.paths.manifest()).await?;
        active_lock.save(self.ctx.paths.lock()).await?;

        let mut ctx_manifest = self.ctx.manifest.write().await;
        *ctx_manifest = manifest;

        let mut ctx_lock = self.ctx.lockfile.write().await;
        *ctx_lock = active_lock;

        Ok(())
    }
}
