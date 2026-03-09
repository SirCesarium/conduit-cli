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
use cli::{Cli, Commands};
use conduit_cli::core::manager::add::models::ModSide as MS;

use crate::cli::commands;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Add { inputs, deps, side } => {
            let core_side = side.map(|s| match s {
                cli::CliModSide::Server => MS::Server,
                cli::CliModSide::Client => MS::Client,
                cli::CliModSide::Both => MS::Both,
            });

            if let Err(e) = commands::add::run(inputs, deps, core_side).await {
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
