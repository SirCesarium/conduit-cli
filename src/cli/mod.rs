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
pub mod progress;
pub mod ui;

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
    /// 🔍 Search for mods on Modrinth
    Search {
        query: String,
        #[arg(
            short,
            long,
            default_value_t = 10,
            help = "Number of results to return"
        )]
        limit: i32,
        #[arg(
            short,
            long,
            default_value_t = 1,
            help = "Page number for paginated results"
        )]
        page: i32,
        #[arg(
            short,
            long,
            default_value = "relevance",
            help = "Sorting method for results"
        )]
        sort: String,
        #[arg(short, long, help = "Facets to filter results")]
        facets: Option<String>,
    },

    /// ➕ Add a new mod to the project using a Modrinth slug or local file path
    #[command(
        alias = "a",
        long_about = "Adds a mod to your project. Supports Modrinth slugs or paths.\n\nExample:\n  conduit add mod-slug\n  conduit add f:./local-mod.jar"
    )]
    Add {
        #[arg(required = true, num_args = 1..)]
        inputs: Vec<String>,

        #[arg(long, num_args = 1.., help = "List of dependencies to add")]
        deps: Vec<String>,

        #[arg(short = 's', long, value_enum)]
        side: Option<CliModSide>,
    },

    /// ✨ Initialize a new conduit project in the current directory
    Init {
        #[arg(short, long, help = "Name of the project")]
        name: Option<String>,

        #[arg(short, long, help = "Loader to use for the project")]
        loader: Option<String>,

        #[arg(short, long, help = "Use default settings without prompts")]
        yes: bool,
    },

    /// 🕷️  Crawl a JAR file to identify its dependencies
    #[command(alias = "crawl")]
    CheckJarDeps { input: String },

    /// 📥 Synchronize and install all mods defined in conduit.toml
    Install {
        #[arg(long, help = "Removes undeclared dependencies in /mods")]
        strict: bool,

        #[arg(long, help = "Rewrites conduit.lock content")]
        force: bool,

        #[arg(short = 'y', long, help = "Skip confirmation prompts")]
        yes: bool,

        #[arg(short = 's', long, value_enum, num_args = 1..)]
        side: Vec<CliModSide>,

        #[arg(short = 'f', long, num_args = 1..)]
        files: Vec<String>,
    },

    /// 🛠️  Verify the integrity of installed mods
    Verify {
        #[command(subcommand)]
        target: Option<VerifyTarget>,
    },

    /// 🗑️  Remove a mod from the project
    Remove { input: String },

    /// 📋 List all currently managed mods
    List,

    /// ⚙️  Install the server loader (NeoForge, etc.) and accept EULA
    #[command(alias = "il")]
    InstallLoader,

    /// ▶️  Runs your server
    Start {
        #[arg(short = 'l', long, help = "Remove progress bars and show all raw logs")]
        show_logs: bool,

        #[arg(short = 'g', long, help = "Enables default Server JAR integrated GUI")]
        show_gui: bool,
    },

    /// 📥 Import a modpack from a .conduit file
    Import {
        #[arg(help = "Path to the modpack .conduit file")]
        input: String,

        #[arg(short = 'y', long, help = "Skip security confirmation prompts")]
        yes: bool,
    },

    /// 📦 Export your project as a shareable modpack
    Export {
        #[arg(help = "Output path for the .conduit file (e.g. my-pack.conduit)")]
        output: String,

        #[arg(
            short = 'c',
            long = "include-config",
            help = "Include the /config folder in the export"
        )]
        include_config: bool,
    },
}
