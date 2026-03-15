use std::path::PathBuf;

use crate::{
    core::{
        engine::{io::conduit::ConduitModpackManager, manager::ProjectManager},
        schemas::modpacks::modrinth::ModrinthIndex,
    },
    errors::ConduitResult,
};

#[derive(Debug, Clone)]
pub enum ModpackType {
    Conduit,
    Mrpack { index: ModrinthIndex },
}

impl ProjectManager {
    pub async fn export(
        &self,
        modpack_type: ModpackType,
        path: PathBuf,
    ) -> ConduitResult<ConduitModpackManager> {
        let manifest = self.ctx.manifest.read().await.clone();
        let lock = self.ctx.lockfile.read().await.clone();
        let include = self.ctx.includefile.read().await.clone();

        let modpack = match modpack_type {
            ModpackType::Conduit => ConduitModpackManager::new(
                path,
                manifest,
                lock,
                include,
                &self.project_root.clone(),
            )?,
            #[allow(unused)]
            ModpackType::Mrpack { index } => {
                unimplemented!();
            }
        };

        Ok(modpack)
    }
}
