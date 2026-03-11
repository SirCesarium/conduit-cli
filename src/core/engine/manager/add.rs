use crate::{
    core::domain::addon::AddonType, core::engine::io::TomlFile,
    core::engine::manager::ProjectManager, errors::ConduitResult, paths::ConduitPaths,
};

impl ProjectManager {
    pub async fn add_addon(&self, identifier: &str, target_type: AddonType) -> ConduitResult<()> {
        let manifest = self.ctx.manifest.read().await;

        self.workflow
            .validate_compatibility(target_type.clone(), &manifest)
            .await?;

        let resolved_list = self
            .resolver
            .resolve_recursively(
                identifier,
                &manifest.project.minecraft,
                &manifest.project.loader,
                target_type,
            )
            .await?;

        let id_map = self.workflow.prepare_addon_uuids(&resolved_list).await?;

        let primary = resolved_list
            .iter()
            .find(|r| r.slug == identifier || r.id == identifier)
            .unwrap_or(&resolved_list[0]);

        self.workflow.update_manifest_addons(primary).await?;

        for resolved in resolved_list {
            self.workflow
                .install_addon_component(resolved, &id_map)
                .await?;
        }

        let lockfile = self.ctx.lockfile.read().await;
        lockfile
            .save(ConduitPaths::get_lock_path(&self.project_root))
            .await?;

        Ok(())
    }
}
