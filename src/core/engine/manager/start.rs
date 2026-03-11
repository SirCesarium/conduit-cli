use crate::{
    engine::manager::ProjectManager,
    errors::{ConduitError, ConduitResult},
};

impl ProjectManager {
    pub async fn start(&self) -> ConduitResult<()> {
        let lockfile = self.ctx.lockfile.read().await;

        let manifest = self.ctx.manifest.read().await;
        if !self.workflow.ensure_loader_presence(&lockfile, &manifest)? {
            return Err(ConduitError::NotInstalled);
        }

        self.workflow.run_server(&lockfile).await
    }
}
