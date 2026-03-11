use crate::{
    core::domain::loader::Loader, core::engine::manager::ProjectManager, errors::ConduitResult,
};

impl ProjectManager {
    pub async fn init(
        &self,
        project_name: String,
        minecraft: String,
        loader: Loader,
    ) -> ConduitResult<()> {
        let manifest = self
            .workflow
            .create_project_manifest(project_name, minecraft, loader)
            .await?;

        let mut ctx_manifest = self.ctx.manifest.write().await;
        *ctx_manifest = manifest;

        Ok(())
    }
}
