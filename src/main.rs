use crate::cli::{AddTarget, Cli, Commands};

mod cli;

#[tokio::main]
async fn main() {
    let cli = Cli::parse_args();

    match cli.command {
        Commands::Init => {}
        Commands::Install => { /* ... */ }
        Commands::Start => { /* ... */ }
        Commands::Add { target, slugs } => {
            let addon_type = match target {
                Some(AddTarget::Mod) => "mod",
                Some(AddTarget::Plugin) => "plugin",
                Some(AddTarget::Datapack) => "datapack",
                None => "auto-detect",
            };

            for slug in slugs {
                // handle installation
                println!("adding {slug} {addon_type}");
            }
        }
    }
}
