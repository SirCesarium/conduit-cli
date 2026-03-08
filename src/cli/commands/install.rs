use crate::cli::ui::CliUi;
use conduit_cli::core::installer::extra_deps::ExtraDepsPolicy;
use conduit_cli::core::installer::project::{InstallProjectOptions, sync_project};
use conduit_cli::core::mods::local::find_missing_local_mods;
use conduit_cli::core::paths::CorePaths;
use conduit_cli::core::modrinth::ModrinthAPI;
use console::style;
use inquire::Confirm;

pub async fn run(
    api: &ModrinthAPI,
    strict: bool,
    force: bool,
    yes: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let paths = CorePaths::from_project_dir(".")?;

    if (strict || force) && !yes {
        println!(
            "{} {}",
            style("WARNING:").yellow().bold(),
            style("This operation can modify files and/or rewrite conduit.lock.").yellow()
        );

        if strict {
            println!(
                "{} {}",
                style("•").yellow(),
                style("--strict will remove unmanaged .jar files from ./mods").yellow()
            );
        }
        if force {
            println!(
                "{} {}",
                style("•").yellow(),
                style("--force will rebuild conduit.lock from conduit.json").yellow()
            );
        }

        let first = Confirm::new("Do you want to continue?")
            .with_default(false)
            .prompt()?;
        if !first {
            return Ok(());
        }

        let second = Confirm::new("Are you absolutely sure?")
            .with_default(false)
            .prompt()?;
        if !second {
            return Ok(());
        }
    }

    let report = find_missing_local_mods(&paths);
    if let Ok(report) = report
        && (!report.missing_files.is_empty() || !report.missing_lock_entries.is_empty())
    {
        println!(
            "{} Missing local mods in ./mods. Add them with:",
            style("✘").red()
        );

        for filename in report.missing_files {
            println!(
                "  {} {}",
                style("conduit add").yellow().bold(),
                style(format!("f:./{filename}")).cyan()
            );
        }

        for key in report.missing_lock_entries {
            println!(
                "  {} {}",
                style("conduit add").yellow().bold(),
                style(format!("f:<path-to-jar>  # for local mod '{key}'")).cyan()
            );
        }

        return Ok(());
    }

    let mut ui = CliUi::new();

    sync_project(
        api,
        &paths,
        &mut ui,
        InstallProjectOptions {
            extra_deps_policy: ExtraDepsPolicy::Callback,
            strict,
            force,
        },
    )
    .await?
    .pruned_files
    .into_iter()
    .for_each(|f| {
        println!(
            "{} Removed unmanaged mod {}",
            style("✘").red(),
            style(f).dim()
        );
    });

    println!("\n{} Project is up to date!", style("✔").green());
    Ok(())
}
