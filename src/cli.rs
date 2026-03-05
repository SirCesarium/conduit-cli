use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "conduit")]
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
    Search {
        query: String,
        #[arg(short, long, default_value_t = 10)]
        limit: i32,
        #[arg(short, long, default_value_t = 1)]
        page: i32,
        #[arg(short, long, default_value = "relevance")]
        sort: String,
        #[arg(short, long)]
        facets: Option<String>,
    },
    #[command(alias = "a")]
    Add {
        input: String,

        #[arg(long, num_args = 1..)]
        deps: Vec<String>,
    },
    Init {
        #[arg(short, long)]
        name: Option<String>,

        #[arg(short, long)]
        loader: Option<String>,

        #[arg(short, long)]
        yes: bool,
    },
    #[command(alias = "crawl")]
    CheckJarDeps {
        input: String,
    },
    Install {
        #[arg(long)]
        strict: bool,

        #[arg(long)]
        force: bool,

        #[arg(short = 'y', long)]
        yes: bool,
    },
    Verify {
        #[command(subcommand)]
        target: Option<VerifyTarget>,
    },
    Remove {
        input: String
    },
    List,
    #[command(alias = "il")]
    InstallLoader,
}
