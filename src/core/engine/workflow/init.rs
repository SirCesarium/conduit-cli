use crate::{
    core::domain::loader::Loader,
    core::engine::io::TomlFile,
    core::engine::workflow::Workflow,
    core::schemas::manifest::Manifest,
    errors::{ConduitError, ConduitResult},
    paths::ConduitPaths,
};

impl Workflow {
    pub async fn create_project_manifest(
        &self,
        project_name: String,
        minecraft: String,
        loader: Loader,
    ) -> ConduitResult<Manifest> {
        let manifest_path = ConduitPaths::get_manifest_path(&self.project_root);

        if manifest_path.exists() {
            return Err(ConduitError::AlreadyInitialized(
                "Project already initialized (conduit.toml exists)".to_string(),
            ));
        }

        let mut manifest = Manifest::default();
        manifest.project.name = project_name;
        manifest.project.minecraft = minecraft;
        manifest.project.loader = loader;

        manifest.save(&manifest_path).await?;

        Ok(manifest)
    }
}
