use clap::ValueEnum;
use clap::{
    Parser, Subcommand,
    builder::{
        Styles,
        styling::{AnsiColor, Effects},
    },
};
use conduit_cli::core::io::project::lock::ModSide as CoreModSide;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, ValueEnum, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum CliModSide {
    Server,
    Client,
    Both,
}

impl From<CliModSide> for CoreModSide {
    fn from(side: CliModSide) -> Self {
        match side {
            CliModSide::Server => CoreModSide::Server,
            CliModSide::Client => CoreModSide::Client,
            CliModSide::Both => CoreModSide::Both,
        }
    }
}

pub mod commands;

fn get_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Yellow.on_default() | Effects::BOLD)
        .usage(AnsiColor::Cyan.on_default() | Effects::BOLD)
        .literal(AnsiColor::Magenta.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Blue.on_default())
}

#[derive(Parser)]
#[command(
    name = "conduit",
    author = env!("CARGO_PKG_AUTHORS"),
    version = env!("CARGO_PKG_VERSION"),
    about = "A lightning-fast Minecraft mod manager built in Rust.",
    styles = get_styles(),
    arg_required_else_help = true,
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Clone, Debug)]
pub enum VerifyTarget {
    Modrinth,
    Local,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(alias = "a")]
    Add {
        #[arg(required = true, num_args = 1..)]
        inputs: Vec<String>,

        #[arg(long, num_args = 1..)]
        deps: Vec<String>,

        #[arg(short = 's', long, value_enum)]
        side: Option<CliModSide>,
    },

    Init {
        #[arg(short, long)]
        name: Option<String>,

        #[arg(short, long)]
        loader: Option<String>,

        #[arg(short, long)]
        yes: bool,
    },
}
