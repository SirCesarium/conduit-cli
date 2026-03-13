use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::cli::commands::{add::AddArgs, init::InitArgs, install::InstallArgs, start::StartArgs};

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
    Add(AddArgs),
    Start(StartArgs),
    Test {
        #[arg(short, long)]
        name: PathBuf,
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
