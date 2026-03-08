use crate::core::events::CoreCallbacks;
use std::path::{Path, PathBuf};

pub mod neoforge;

pub struct LoaderInfo {
    pub name: String,
    pub version: String,
}

impl LoaderInfo {
    pub fn parse(loader_str: &str) -> Self {
        let mut parts = loader_str.split('@');
        let name = parts.next().unwrap_or("").to_string();
        let version = parts.next().unwrap_or("latest").to_string();
        Self { name, version }
    }
}

pub enum LoaderType {
    NeoForge,
}

impl LoaderType {
    pub async fn get_latest_version(
        &self,
        mc_version: &str,
        callbacks: &mut dyn CoreCallbacks,
    ) -> Result<String, Box<dyn std::error::Error>> {
        match self {
            Self::NeoForge => neoforge::get_latest_neoforge_version(mc_version, callbacks).await,
        }
    }

    pub async fn download_installer(
        &self,
        mc_version: &str,
        loader_version: &str,
        install_path: &Path,
        callbacks: &mut dyn CoreCallbacks,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        match self {
            Self::NeoForge => {
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
        installer_path: &Path,
        callbacks: &mut dyn CoreCallbacks,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        match self {
            Self::NeoForge => neoforge::execute_neoforge_installer(installer_path, callbacks).await,
        }
    }

    pub fn post_install(
        &self,
        installer_path: &Path,
        install_path: &Path,
        callbacks: &mut dyn CoreCallbacks,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::NeoForge => {
                neoforge::post_install_neoforge(installer_path, install_path, callbacks)
            }
        }
    }
}
