use crate::api::ApiError;
use crate::core::resolver::Resolver;
use crate::domain::loader::Loader;

pub struct ResolvedLoader {
    pub url: String,
    pub hash: String,
    pub file_name: String,
}

impl Resolver {
    pub async fn resolve_loader(&self, loader: &Loader) -> Result<ResolvedLoader, ApiError> {
        match loader {
            Loader::Vanilla { version } => {
                let url = self.ctx.api.mojang.get_server_url(version).await?;
                Ok(ResolvedLoader {
                    url,
                    hash: String::new(),
                    file_name: "server.jar".to_string(),
                })
            }
            Loader::Paper { version } => {
                let build = self.ctx.api.papermc.get_latest_build(version).await?;
                Ok(ResolvedLoader {
                    url: self.ctx.api.papermc.build_download_url(
                        version,
                        build.build,
                        &build.downloads.application.name,
                    ),
                    hash: build.downloads.application.sha256,
                    file_name: "server.jar".to_string(),
                })
            }
            Loader::Purpur { version } => {
                let build = self.ctx.api.purpurmc.get_latest_build(version).await?;
                Ok(ResolvedLoader {
                    url: self.ctx.api.purpurmc.build_download_url(version, &build),
                    hash: String::new(),
                    file_name: "server.jar".to_string(),
                })
            }
            Loader::Fabric { version: _mc_version } => {
                let installer_version = self.ctx.api.fabricmc.get_latest_installer().await?;
                let url = self.ctx.api.fabricmc.build_installer_url(&installer_version);
                Ok(ResolvedLoader {
                    url,
                    hash: String::new(),
                    file_name: "server-installer.jar".to_string(),
                })
            }
            Loader::Forge { version } => {
                let forge_version = self.ctx.api.minecraftforge.get_latest_version(version).await?;
                let url = self.ctx.api.minecraftforge.build_bin_url(&forge_version, "installer");
                Ok(ResolvedLoader {
                    url,
                    hash: String::new(),
                    file_name: "server-installer.jar".to_string(),
                })
            }
            Loader::Neoforge { version } => {
                let url = self.ctx.api.neoforged.build_bin_url(version, "installer");
                Ok(ResolvedLoader {
                    url,
                    hash: String::new(),
                    file_name: "server-installer.jar".to_string(),
                })
            }
        }
    }
}
