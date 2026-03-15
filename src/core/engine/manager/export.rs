use std::path::PathBuf;

use crate::{
    core::{
        engine::{
            io::{IncludeFile, conduit::ConduitModpackManager},
            manager::ProjectManager,
        },
        schemas::{include::ConduitInclude, modpacks::modrinth::ModrinthIndex},
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

        let modpack = match modpack_type {
            ModpackType::Conduit => {
                let ignore = ConduitInclude::load(self.ctx.paths.include()).await?;
                ConduitModpackManager::new(
                    path,
                    manifest,
                    lock,
                    ignore,
                    &self.project_root.clone(),
                )?
            }
            #[allow(unused)]
            ModpackType::Mrpack { index } => {
                unimplemented!();
            }
        };

        Ok(modpack)
    }
}
