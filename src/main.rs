use miette::IntoDiagnostic;
use std::env;
use std::sync::Arc;

use conduit_cli::core::engine::io::TomlFile;
use conduit_cli::core::schemas::lock::Lockfile;
use conduit_cli::core::schemas::manifest::Manifest;
use conduit_cli::{core::engine::ConduitContext, paths::ConduitPaths};

mod cli;
use crate::cli::{Cli, Commands, commands::Cmds};

#[tokio::main]
async fn main() {
    miette::set_hook(Box::new(|_| {
        Box::new(
            miette::MietteHandlerOpts::new()
                .terminal_links(true)
                .unicode(true)
                .context_lines(3)
                .build(),
        )
    }))
    .expect("Failed to setup miette");

    if let Err(e) = run_app().await {
        eprintln!("{e:?}");
        std::process::exit(1);
    }
}

async fn run_app() -> miette::Result<()> {
    let cli = Cli::parse_args();
    let current_dir = env::current_dir().into_diagnostic()?;
    let store_dir = ConduitPaths::get_store_dir();

    ConduitPaths::ensure_dirs().into_diagnostic()?;

    let manifest_path = ConduitPaths::get_manifest_path(&current_dir);
    let lock_path = ConduitPaths::get_lock_path(&current_dir);

    let manifest = if manifest_path.exists() {
        Manifest::load(&manifest_path).await.into_diagnostic()?
    } else {
        Manifest::default()
    };

    let lockfile = if lock_path.exists() {
        Lockfile::load(&manifest_path).await.into_diagnostic()?
    } else {
        Lockfile::default()
    };

    let ctx = Arc::new(ConduitContext::new(store_dir, manifest, lockfile));
    let cmds = Cmds::new(ctx, current_dir);

    match cli.command {
        Commands::Init(args) => {
            cmds.init(args).await?;
        }
        _ => {
            println!("Not implemented");
        }
    }

    Ok(())
}
