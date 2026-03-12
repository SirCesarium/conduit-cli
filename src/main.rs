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
    let paths = ConduitPaths::new(current_dir);

    paths.ensure_dirs().into_diagnostic()?;

    let manifest_path = paths.manifest();
    let lock_path = paths.lock();

    let manifest = if manifest_path.exists() {
        Manifest::load(&manifest_path).await.into_diagnostic()?
    } else {
        Manifest::default()
    };

    let lockfile = if lock_path.exists() {
        Lockfile::load(&lock_path).await.into_diagnostic()?
    } else {
        let mut lf = Lockfile::default();

        lf.instance
            .minecraft_version
            .clone_from(&manifest.project.minecraft);

        lf.instance.loader = manifest.project.loader.clone();

        lf
    };

    let ctx = Arc::new(ConduitContext::new(paths.clone(), manifest, lockfile));
    let cmds = Cmds::new(ctx, paths.root);

    match cli.command {
        Commands::Init(args) => {
            cmds.init(args).await?;
        }
        Commands::Install(args) => {
            cmds.install(args).await?;
        }
        Commands::Add(args) => {
            cmds.add(args).await?;
        }
        Commands::Start => {
            println!("Not implemented");
        }
    }

    Ok(())
}
