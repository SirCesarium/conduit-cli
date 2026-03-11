use clap::{Parser, Subcommand};

use crate::cli::commands::{init::InitArgs, install::InstallArgs};

pub mod commands;
mod errors;
mod ui;

#[derive(Parser)]
#[command(name = "conduit")]
#[command(about = "A lightning-fast Minecraft mod manager", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Init(InitArgs),
    Install(InstallArgs),
    Start,
    Add {
        #[command(subcommand)]
        target: Option<AddTarget>,

        #[arg(required = true, num_args = 1..)]
        slugs: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum AddTarget {
    Mod,
    Plugin,
    Datapack,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
