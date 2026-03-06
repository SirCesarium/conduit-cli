use crate::core::events::CoreCallbacks;
use crate::core::io::project::ProjectFiles;
use crate::core::paths::CorePaths;
use crate::server_launcher::ServerLauncher;
use std::error::Error;

pub async fn start_server(
    paths: &CorePaths,
    callbacks: &mut dyn CoreCallbacks,
    show_logs: bool,
    show_gui: bool,
) -> Result<(), Box<dyn Error>> {
    let config = ProjectFiles::load_manifest(paths)?;

    let launcher = match config.loader.to_lowercase().as_str() {
        s if s.contains("neoforge") => ServerLauncher::Neoforge,
        _ => return Err(format!("Unsupported loader for launching: {}", config.loader).into()),
    };

    launcher
        .launch(
            paths.project_dir().to_path_buf(),
            show_logs,
            show_gui,
            callbacks,
        )
        .await;

    Ok(())
}
