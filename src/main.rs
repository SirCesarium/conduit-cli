use clap::Parser;
use console::style;

mod cli;
mod commands;
mod progress;
mod ui;
use cli::{Cli, Commands, VerifyTarget};
use conduit_cli::modrinth::ModrinthAPI;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let api = ModrinthAPI::new();

    match cli.command {
        Commands::Search {
            query,
            limit,
            page,
            sort,
            facets,
        } => {
            if let Err(e) = commands::search::run(&api, query, limit, page, sort, facets).await {
                eprintln!("{} {}", style("Error:").red().bold(), e);
            }
        }
        Commands::Add { input, deps } => {
            if let Err(e) = commands::add::run(&api, input, deps).await {
                eprintln!("{} {}", style("Error:").red().bold(), e);
            }
        }
        Commands::Init { name, loader, yes } => {
            if let Err(e) = commands::init::run(name, loader, yes) {
                eprintln!("{} {}", style("Error:").red().bold(), e);
            }
        }
        Commands::CheckJarDeps { input } => {
            if let Err(e) = commands::check_jar_deps::run(input) {
                eprintln!("{} {}", style("Error:").red().bold(), e);
            }
        }
        Commands::Install { strict, force, yes } => {
            if let Err(e) = commands::install::run(&api, strict, force, yes).await {
                eprintln!("{} {}", style("Error:").red().bold(), e);
            }
        }
        Commands::Verify { target } => {
            let target = target.unwrap_or(VerifyTarget::Modrinth);
            if let Err(e) = commands::verify::run(target).await {
                eprintln!("{} {}", style("Error:").red().bold(), e);
            }
        }
        Commands::Remove { input } => {
            if let Err(e) = commands::remove::run(input).await {
                eprintln!("{} {}", style("Error:").red().bold(), e);
            }
        }
        Commands::List => {
            if let Err(e) = commands::list::run(&api).await {
                eprintln!("{} {}", style("Error:").red().bold(), e);
            }
        }
        Commands::InstallLoader => {
            if let Err(e) = commands::install_loader::run().await {
                eprintln!("{} {}", style("Error:").red().bold(), e);
            }
        }
    }
}
