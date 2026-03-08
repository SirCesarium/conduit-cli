use crate::core::events::CoreCallbacks;
use crate::core::io::project::ProjectFiles;
use crate::core::runtime::loaders;
use crate::core::paths::CorePaths;
use crate::core::runtime::launchers::ServerLauncher;
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
        .ok_or("No loader version found in lock file")?;

    let loader_info = loaders::LoaderInfo::parse(&loader_raw);
    let loader_version = loader_info.version;

    let launcher = match loader_info.name.to_lowercase().as_str() {
        "neoforge" => ServerLauncher::Neoforge,
        "vanilla" => ServerLauncher::Vanilla,
        _ => return Err(format!("Unsupported loader: {}", loader_info.name).into()),
    };

    if !launcher.is_ready(paths, &loader_version) {
        return Err(format!(
            "Loader {} version {} is not installed or files are missing.",
            loader_info.name, loader_version
        )
        .into());
    }

    let config = crate::core::io::server::config::ServerConfig::load(paths.config_path())?;

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
