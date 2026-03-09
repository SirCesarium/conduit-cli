use conduit_cli::core::installer::extra_deps::ExtraDepsPolicy;
use conduit_cli::core::installer::project::{InstallProjectOptions, add_mods_to_project};
use conduit_cli::core::io::project::lock::ModSide;
use conduit_cli::core::modrinth::ModrinthAPI;
use conduit_cli::core::paths::CorePaths;
use console::style;

use crate::cli::ui::CliUi;

pub async fn run(
    api: &ModrinthAPI,
    inputs: Vec<String>,
    deps: Vec<String>,
    explicit_side: Option<ModSide>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !deps.is_empty() && inputs.len() > 1 {
        return Err("The --deps flag can only be used when adding a single mod.".into());
    }

    let paths = CorePaths::from_project_dir(".")?;
    let mut ui = CliUi::new();

    let result = add_mods_to_project(
        api,
        &paths,
        inputs,
        deps,
        &mut ui,
        InstallProjectOptions {
            extra_deps_policy: ExtraDepsPolicy::Callback,
            ..Default::default()
        },
        explicit_side
    )
    .await;

    match result {
        Ok(()) => {
            println!("{} Project updated successfully.", style("✔").green());
            Ok(())
        }
        Err(e) => match e {
            conduit_cli::core::error::CoreError::ProjectNotFound { slug } => {
                let suggestions = api.get_suggestions(&slug).await;

                println!(
                    "{} Project not found: {}",
                    style("✘").red(),
                    style(&slug).yellow().bold()
                );
                ui.print_suggestions(&suggestions);

                Ok(())
            }
            _ => Err(Box::new(e)),
        },
    }
}
