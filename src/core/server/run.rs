use crate::core::events::CoreCallbacks;
use crate::core::io::project::ProjectFiles;
use crate::core::io::server::config::ServerConfig;
use crate::core::paths::CorePaths;
use crate::core::runtime::launchers::ServerLauncher;
use crate::core::runtime::loaders::{self, LoaderType};
use std::error::Error;

pub async fn start_server(
    paths: &CorePaths,
    callbacks: &mut dyn CoreCallbacks,
    show_logs: bool,
    show_gui: bool,
) -> Result<(), Box<dyn Error>> {
    let lock = ProjectFiles::load_lock(paths)?;

    let loader_raw = lock
        .loader_version
        .as_ref()
        .ok_or("No loader version found in lock file")?;

    let loader_info = loaders::LoaderInfo::parse(loader_raw);
    let loader_version = loader_info.version;

    let (loader_type, launcher) = match loader_info.name.to_lowercase().as_str() {
        "neoforge" => (LoaderType::NeoForge, ServerLauncher::Neoforge),
        "vanilla" => (LoaderType::Vanilla, ServerLauncher::Vanilla),
        _ => return Err(format!("Unsupported loader: {}", loader_info.name).into()),
    };

    if !paths.is_loader_ready(&loader_type, &loader_version) {
        return Err(format!(
            "Loader {} version {} is not installed or files are missing. Run setup first.",
            loader_info.name, loader_version
        )
        .into());
    }

    let config = ServerConfig::load(paths.config_path())?;

    launcher
        .launch(
            paths,
            &config,
            &loader_version,
            show_logs,
            show_gui,
            callbacks,
        )
        .await;

    Ok(())
}
