use crate::cli::ui::CliUi;
use conduit_cli::core::{
    io::{project::ProjectFiles, server::config::ServerConfig},
    paths::CorePaths,
    runtime::{launchers::ServerLauncher, loaders::LoaderInfo},
    server::run::start_server,
};
use console::style;
use std::error::Error;

pub async fn run(show_logs: bool, show_gui: bool) -> Result<(), Box<dyn Error>> {
    let paths = CorePaths::from_project_dir(".")?;
    let mut ui = CliUi::new();

    let config = match ServerConfig::load_or_create(paths.config_path()) {
        Ok(cfg) => cfg,
        Err(e) => {
            println!("{} {}", style("✘").red(), e);
            return Err(e.into());
        }
    };

    let Ok(lock) = ProjectFiles::load_lock(&paths) else {
        println!(
            "{} No {} found. Please run {} first.",
            style("!").yellow(),
            style("conduit.lock").bold(),
            style("conduit install").cyan()
        );
        return Ok(());
    };

    let loader_raw = lock
        .loader_version
        .ok_or("No loader version found in lock file")?; // FIXME: probably the loader isn't installed
    let loader_info = LoaderInfo::parse(&loader_raw);
    let loader_version = loader_info.version;

    let launcher = if loader_info.name.to_lowercase().as_str() == "neoforge" {
        ServerLauncher::Neoforge
    } else {
        println!(
            "{} Unsupported loader: {}",
            style("✘").red(),
            loader_info.name
        );
        return Ok(());
    };

    if !launcher.is_ready(&paths, &loader_version) {
        println!(
            "{} Loader {} version {} is not installed.",
            style("!").yellow(),
            style(&loader_info.name).bold(),
            style(&loader_version).cyan()
        );

        // TODO: auto call install loader
        println!("  Run {} to fix this.", style("conduit install").cyan());
        return Ok(());
    }

    let properties_path = paths.project_dir().join("server.properties");
    if properties_path.exists() {
        if let Err(e) = config.patch_properties(&properties_path) {
            println!(
                "{} Failed to patch server.properties: {}",
                style("!").yellow(),
                e
            );
        } else {
            println!(
                "{} Configuration synchronized with {}",
                style("✔").green(),
                style("server.properties").dim()
            );
        }
    }

    start_server(&paths, &mut ui, show_logs, show_gui).await
}
