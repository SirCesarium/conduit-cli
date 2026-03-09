use super::models::AddRequest;
use crate::core::error::CoreResult;
use crate::core::installer::download::download_and_link;
use crate::core::installer::resolve::resolve_remote_mod;
use crate::core::io::project::lock::{LockedMod, ModSide};
use crate::core::manager::ProjectManager;
use crate::core::manager::add::models::{RemoteSource, ResourceSource};

impl ProjectManager {
    pub(crate) async fn handle_add_mod(&mut self, request: AddRequest) -> CoreResult<()> {
        match request.source {
            ResourceSource::Remote(source) => {
                self.install_remote(source, false).await?;
                self.ctx.save_state()?;
            }
            ResourceSource::Local(path) => {
                self.install_local_mod(path, request.is_dependency).await?;
            }
        }
        Ok(())
    }

    async fn install_remote(
        &mut self,
        source: RemoteSource,
        is_dependency: bool,
    ) -> CoreResult<()> {
        if self.ctx.lock.is_mod_installed(source.slug()) {
            return Ok(());
        }

        let minecraft_version = &self.ctx.manifest.mc_version;
        let loader_info = &self.ctx.manifest.get_loader_info();

        let r#mod = resolve_remote_mod(&self.ctx, &source, minecraft_version, loader_info).await?;

        download_and_link(&self.ctx, &r#mod.download_url, &r#mod.filename, &r#mod.hash).await?;

        if !is_dependency {
            let version_to_save = match &source {
                RemoteSource::Modrinth { version, .. } => {
                    version.clone().unwrap_or_else(|| "latest".to_string())
                }
            };
            self.ctx
                .manifest
                .mods
                .insert(r#mod.slug.clone(), version_to_save);
        }

        let locked = LockedMod {
            id: r#mod.id,
            version_id: r#mod.version_id,
            filename: r#mod.filename,
            url: r#mod.download_url,
            hash: r#mod.hash,
            side: ModSide::Both,
            dependencies: r#mod.deps.iter().map(|d| d.slug().to_string()).collect(),
        };

        self.ctx.lock.locked_mods.insert(r#mod.slug.clone(), locked);

        for dep_source in r#mod.deps {
            if !self.ctx.lock.locked_mods.contains_key(dep_source.slug()) {
                Box::pin(self.install_remote(dep_source, true)).await?;
            }
        }

        Ok(())
    }

    #[allow(clippy::unused_async)]
    async fn install_local_mod(
        &mut self,
        _path: std::path::PathBuf,
        _is_dep: bool,
    ) -> CoreResult<()> {
        dbg!("Feature not implemented yet!");
        Ok(())
    }
}
