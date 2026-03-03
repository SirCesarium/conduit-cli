use clap::Parser;
use console::style;

mod cli;
mod commands;
mod config;
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
        Commands::Install { input } => {
            if let Err(e) = commands::install::run(&api, input).await {
                eprintln!("{} {}", style("Error:").red().bold(), e);
            }
        }
        Commands::Init { name, loader, yes } => {
            if let Err(e) = commands::init::run(name, loader, yes) {
                eprintln!("{} {}", style("Error:").red().bold(), e);
            }
        }
    }
}
