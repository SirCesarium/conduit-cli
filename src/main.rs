use clap::Parser;
use console::style;

mod cli;
mod commands;
mod config;
mod inspector;
mod lock;
mod modrinth;
mod progress;
use cli::{Cli, Commands};
use modrinth::ModrinthAPI;

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
        Commands::Add { input } => {
            if let Err(e) = commands::add::run(&api, input).await {
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
        Commands::Install => {
            if let Err(e) = commands::install::run(&api).await {
                eprintln!("{} {}", style("Error:").red().bold(), e);
            }
        }
    }
}
