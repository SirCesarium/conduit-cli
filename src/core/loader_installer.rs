use crate::core::events::CoreCallbacks;
use crate::core::io::load_config;
use crate::core::paths::CorePaths;
use crate::loaders::{Loader, LoaderInfo};
use std::error::Error;

pub async fn install_loader(
    paths: &CorePaths,
    callbacks: &mut dyn CoreCallbacks,
) -> Result<(), Box<dyn Error>> {
    let config = load_config(paths)?;
    let loader_info = LoaderInfo::parse(&config.loader);
    let loader = match loader_info.name.as_str() {
        "neoforge" => Loader::NeoForge,
        _ => return Err(format!("Unsupported loader: {}", loader_info.name).into()),
    };

    let loader_version = if loader_info.version == "latest" {
        loader
            .get_latest_version(&config.mc_version, callbacks)
            .await?
    } else {
        loader_info.version.clone()
    };

    let loader_dir = loader
        .download_installer(
            &config.mc_version,
            &loader_version,
            &paths.project_dir(),
            callbacks,
        )
        .await?;

    let installer_path = loader.execute_installer(&loader_dir, callbacks).await?;

    loader.post_install(&installer_path, &paths.project_dir(), callbacks).await?;

    Ok(())
}
