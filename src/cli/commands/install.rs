use crate::cli::ui::CliUi;
use conduit_cli::core::error::CoreError;
use conduit_cli::core::installer_old::extra_deps::ExtraDepsPolicy;
use conduit_cli::core::installer_old::project::{InstallProjectOptions, sync_project};
use conduit_cli::core::io::project::ProjectFiles;
use conduit_cli::core::io::project::lock::ModSide;
use conduit_cli::core::apis::modrinth::ModrinthAPI;
use conduit_cli::core::paths::CorePaths;
use console::style;
use inquire::Confirm;
use std::fmt::Write;
use std::fs;
use std::path::PathBuf;

#[allow(clippy::too_many_lines)]
pub async fn run(
    api: &ModrinthAPI,
    strict: bool,
    force: bool,
    yes: bool,
    sides: Vec<ModSide>,
    provided_files: Vec<String>,
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

    if !provided_files.is_empty() {
        let lock = ProjectFiles::load_lock(&paths)?;
        fs::create_dir_all(paths.mods_dir())?;

        for entry in provided_files {
            if let Some((input_slug, path_str)) = entry.rsplit_once(':') {
                let locked = lock
                    .locked_mods
                    .get(input_slug)
                    .or_else(|| lock.locked_mods.get(&format!("local:{input_slug}")));

                if let Some(locked) = locked {
                    let src = PathBuf::from(path_str);
                    let dest = paths.mods_dir().join(&locked.filename);

                    if src.exists() {
                        fs::copy(&src, &dest)?;
                        println!(
                            "{} Repaired local mod: {}",
                            style("✔").green(),
                            style(&locked.filename).cyan()
                        );
                    } else {
                        println!(
                            "{} File not found at: {}",
                            style("✘").red(),
                            style(path_str).dim()
                        );
                    }
                } else {
                    println!(
                        "{} Mod '{}' (or 'local:{}') not found in lock",
                        style("⚠").yellow(),
                        input_slug,
                        input_slug
                    );
                }
            } else {
                println!(
                    "{} Use format: slug:path (e.g., create:./file.jar)",
                    style("!").yellow()
                );
            }
        }
    }

    let mut ui = CliUi::new();

    let result = sync_project(
        api,
        &paths,
        &mut ui,
        InstallProjectOptions {
            extra_deps_policy: ExtraDepsPolicy::Callback,
            strict,
            force,
            allowed_sides: sides,
        },
    )
    .await;

    match result {
        Ok(report) => {
            report.pruned_files.into_iter().for_each(|f| {
                println!(
                    "{} Removed unmanaged mod {}",
                    style("✘").red(),
                    style(f).dim()
                );
            });
            println!("\n{} Project is up to date!", style("✔").green());
        }
        Err(CoreError::MissingLocalFiles { mods }) => {
            println!(
                "\n{} {}",
                style("✘").red().bold(),
                style("Missing local mods in ./mods:").bold()
            );

            let mut help_example = String::new();

            for (slug, filename) in &mods {
                println!(
                    "  - {} {}",
                    style(slug).cyan(),
                    style(format!("({filename})")).dim()
                );

                let clean_slug = slug.strip_prefix("local:").unwrap_or(slug);

                let _ = write!(help_example, " {clean_slug}:./path/to/{filename}");
            }

            println!(
                "\n{}",
                style("To fix this, provide the files using:").yellow()
            );
            println!(
                "  {}",
                style(format!("conduit install --files{help_example}"))
                    .cyan()
                    .bold()
            );

            return Err(Box::new(CoreError::MissingLocalFiles { mods }));
        }
        Err(e) => return Err(Box::new(e)),
    }

    Ok(())
}
