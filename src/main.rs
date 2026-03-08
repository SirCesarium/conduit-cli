#![warn(clippy::pedantic)]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::module_name_repetitions,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::missing_const_for_fn,
    clippy::doc_markdown
)]

use clap::Parser;
use console::style;

mod cli;
use cli::{Cli, Commands, VerifyTarget};
use conduit_cli::core::modrinth::ModrinthAPI;

use crate::cli::commands;

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
        Commands::Add { inputs, deps } => {
            if let Err(e) = commands::add::run(&api, inputs, deps).await {
                eprintln!("{} {}", style("Error:").red().bold(), e);
            }
        }
        Commands::Init { name, loader, yes } => {
            if let Err(e) = commands::init::run(name, loader, yes) {
                eprintln!("{} {}", style("Error:").red().bold(), e);
            }
        }
        Commands::CheckJarDeps { input } => {
            if let Err(e) = commands::check_jar_deps::run(&input) {
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
            if let Err(e) = commands::verify::run(&target) {
                eprintln!("{} {}", style("Error:").red().bold(), e);
            }
        }
        Commands::Remove { input } => {
            if let Err(e) = commands::remove::run(&input) {
                eprintln!("{} {}", style("Error:").red().bold(), e);
            }
        }
        Commands::List => {
            if let Err(e) = commands::list::run(&api) {
                eprintln!("{} {}", style("Error:").red().bold(), e);
            }
        }
        Commands::InstallLoader => {
            if let Err(e) = commands::install_loader::run().await {
                eprintln!("{} {}", style("Error:").red().bold(), e);
            }
        }
        Commands::Start {
            show_logs,
            show_gui,
        } => {
            if let Err(e) = commands::start::run(show_logs, show_gui).await {
                eprintln!("{} {}", style("Error:").red().bold(), e);
            }
        }

        Commands::Import { input, yes } => {
            if let Err(e) = commands::import::run(&input, yes) {
                eprintln!("{} {}", style("Error:").red().bold(), e);
            }
        }
        Commands::Export {
            output,
            include_config,
        } => {
            if let Err(e) = commands::export::run(&output, include_config) {
                eprintln!("{} {}", style("Error:").red().bold(), e);
            }
        }
    }
}
