use futures_util::stream::{self, StreamExt};
use std::sync::Arc;

use crate::{
    core::{
        domain::addon::AddonType,
        engine::{io::TomlFile, manager::ProjectManager, resolver::addon::ResolvedAddon},
    },
    errors::ConduitResult,
};

impl ProjectManager {
    pub async fn add_addons(
        &self,
        identifiers: Vec<String>,
        target_type: AddonType,
    ) -> ConduitResult<()> {
        let (mc_version, loader) = {
            let manifest = self.ctx.manifest.read().await;
            self.workflow
                .validate_compatibility(&target_type, &manifest)?;
            (
                manifest.project.minecraft.clone(),
                manifest.project.loader.clone(),
            )
        };

        let resolved_results = stream::iter(identifiers.clone())
            .map(|id| {
                let resolver = &self.resolver;
                let mc = mc_version.clone();
                let lo = loader.clone();
                let tt = target_type.clone();
                async move { resolver.resolve_recursively(&id, &mc, &lo, tt).await }
            })
            .buffer_unordered(4)
            .collect::<Vec<ConduitResult<Vec<ResolvedAddon>>>>()
            .await;

        let mut all_resolved = Vec::new();
        for res in resolved_results {
            all_resolved.extend(res?);
        }

        let id_map = Arc::new(self.workflow.prepare_addon_id(&all_resolved).await?);

        for id in &identifiers {
            if let Some(primary) = all_resolved.iter().find(|r| &r.slug == id || &r.id == id) {
                self.workflow.update_manifest_addons(primary).await?;
            }
        }

        let workflow = Arc::new(&self.workflow);
        stream::iter(all_resolved)
            .map(|resolved| {
                let w = Arc::clone(&workflow);
                let ids = Arc::clone(&id_map);
                async move { w.install_addon_component(resolved, &ids).await }
            })
            .buffer_unordered(8)
            .collect::<Vec<ConduitResult<()>>>()
            .await
            .into_iter()
            .collect::<ConduitResult<Vec<()>>>()?;

        let lock_path = self.ctx.paths.lock();
        let lockfile = self.ctx.lockfile.read().await;
        lockfile.save(lock_path).await?;

        Ok(())
    }
}
