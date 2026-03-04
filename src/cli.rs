use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "conduit")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
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
    Install,
    Remove {
        input: String
    },
    List,
}
