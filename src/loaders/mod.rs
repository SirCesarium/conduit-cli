use crate::core::events::CoreCallbacks;

pub mod neoforge;

pub struct LoaderInfo {
    pub name: String,
    pub version: String,
}

impl LoaderInfo {
    pub fn parse(loader_str: &str) -> Self {
        let parts: Vec<&str> = loader_str.split('@').collect();
        let name = parts[0].to_string();
        let version = if parts.len() > 1 {
            parts[1].to_string()
        } else {
            "latest".to_string()
        };
        Self { name, version }
    }
}

pub enum Loader {
    NeoForge,
}

impl Loader {
    pub async fn get_latest_version(
        &self,
        mc_version: &str,
        callbacks: &mut dyn CoreCallbacks,
    ) -> Result<String, Box<dyn std::error::Error>> {
        match self {
            Loader::NeoForge => neoforge::get_latest_neoforge_version(mc_version, callbacks).await,
        }
    }

    pub async fn download_installer(
        &self,
        mc_version: &str,
        loader_version: &str,
        install_path: &std::path::Path,
        callbacks: &mut dyn CoreCallbacks,
    ) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
        match self {
            Loader::NeoForge => {
                neoforge::download_neoforge_installer(
                    mc_version,
                    loader_version,
                    install_path,
                    callbacks,
                )
                .await
            }
        }
    }

    pub async fn execute_installer(
        &self,
        installer_path: &std::path::Path,
        callbacks: &mut dyn CoreCallbacks,
    ) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
        match self {
            Loader::NeoForge => {
                neoforge::execute_neoforge_installer(installer_path, callbacks).await
            }
        }
    }

    pub async fn post_install(
        &self,
        installer_path: &std::path::Path,
        install_path: &std::path::Path,
        callbacks: &mut dyn CoreCallbacks,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Loader::NeoForge => {
                neoforge::post_install_neoforge(installer_path, install_path, callbacks).await
            }
        }
    }
}
