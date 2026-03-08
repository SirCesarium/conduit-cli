use crate::core::error::{CoreError, CoreResult};
use crate::core::events::CoreCallbacks;
use crate::core::io::project::{InstanceType, ProjectFiles};
use crate::core::paths::CorePaths;
use crate::core::runtime::loaders::{LoaderInfo, LoaderType};

pub async fn install_loader(
    paths: &CorePaths,
    callbacks: &mut dyn CoreCallbacks,
) -> CoreResult<()> {
    let config = ProjectFiles::load_manifest(paths)?;

    if let InstanceType::Client = config.instance_type {
        return Err(CoreError::ServerOnlyFeature);
    }

    let loader_info = LoaderInfo::parse(&config.loader);

    let loader = match loader_info.name.as_str() {
        "neoforge" => LoaderType::NeoForge,
        _ => return Err(CoreError::UnsupportedLoader(loader_info.name)),
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
            paths.project_dir(),
            callbacks,
        )
        .await?;

    let installer_path = loader.execute_installer(&loader_dir, callbacks).await?;

    loader.post_install(&installer_path, paths.project_dir(), callbacks)?;

    let mut lock = ProjectFiles::load_lock(paths)?;

    lock.loader_version = Some(format!("{}@{}", loader_info.name, loader_version));

    ProjectFiles::save_lock(paths, &lock)?;

    Ok(())
}
