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

        validate_loader_compatibility(&manifest.project.minecraft, &loader)?;

        let resolved = self
            .resolver
            .resolve_loader(&loader, &manifest.project.minecraft)
            .await?;

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

fn validate_loader_compatibility(mc_version: &str, loader: &Loader) -> ConduitResult<()> {
    match loader {
        Loader::Forge { version } => {
            if !version.contains(mc_version) {
                return Err(crate::errors::ConduitError::Validation(format!(
                    "Forge {version} is not compatible with Minecraft {mc_version}"
                )));
            }
        }
        Loader::Neoforge { version } => {
            let mc_parts: Vec<&str> = mc_version.split('.').collect();
            let nf_parts: Vec<&str> = version.split('.').collect();

            if let (Some(mc_minor), Some(nf_major)) = (mc_parts.get(1), nf_parts.first())
                && mc_minor != nf_major
            {
                return Err(crate::errors::ConduitError::Validation(format!(
                    "NeoForge {version} is not compatible with Minecraft {mc_version}"
                )));
            }
        }
        _ => {} // Vanilla, Fabric, etc., se validan en el resolver
    }
    Ok(())
}
