use crate::cli::commands;
use crate::cli::ui::CliUi;
use conduit_cli::core::apis::modrinth::ModrinthAPI;
use conduit_cli::core::{
    io::{project::ProjectFiles, server::config::ServerConfig},
    paths::CorePaths,
    runtime::loaders::{LoaderInfo, LoaderType},
    server::run::start_server,
};
use console::style;
use std::error::Error;

pub async fn run(show_logs: bool, show_gui: bool) -> Result<(), Box<dyn Error>> {
    let paths = CorePaths::from_project_dir(".")?;
    let api = ModrinthAPI::new();
    let mut ui = CliUi::new();

    let config = match ServerConfig::load_or_create(paths.config_path()) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("{} {}", style("✘").red(), e);
            return Err(e.into());
        }
    };

    let manifest = ProjectFiles::load_manifest(&paths)?;

    if !paths.lock_path().exists() {
        println!(
            "{} No lockfile found. Synchronizing project...",
            style("!").blue()
        );

        let sides = manifest.instance_type.allowed_sides();
        commands::install::run(&api, false, false, true, sides, vec![]).await?;
    }

    let mut lock = ProjectFiles::load_lock(&paths)?;

    if lock.loader.is_none() {
        println!(
            "{} Loader not configured. Installing loader...",
            style("!").blue()
        );
        commands::install_loader::run().await?;
        lock = ProjectFiles::load_lock(&paths)?;
    }

    let loader_raw = lock
        .loader
        .clone()
        .ok_or("Critical: Loader version missing after sync")?;

    let loader_info = LoaderInfo::parse(&loader_raw);
    let loader_version = loader_info.version;

    let loader_type = match loader_info.name.to_lowercase().as_str() {
        "neoforge" => LoaderType::NeoForge,
        "vanilla" => LoaderType::Vanilla,
        _ => {
            eprintln!(
                "{} Unsupported loader: {}",
                style("✘").red(),
                loader_info.name
            );
            return Ok(());
        }
    };

    if !paths.is_loader_ready(&loader_type, &loader_version) {
        println!("{} Loader binary missing. Installing...", style("!").blue());
        commands::install_loader::run().await?;
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
