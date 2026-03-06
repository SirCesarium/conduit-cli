use crate::ui::CliUi;
use conduit_cli::core::installer::extra_deps::ExtraDepsPolicy;
use conduit_cli::core::installer::project::{InstallProjectOptions, add_mods_to_project};
use conduit_cli::core::paths::CorePaths;
use conduit_cli::modrinth::ModrinthAPI;
use console::style;

pub async fn run(
    api: &ModrinthAPI,
    inputs: Vec<String>,
    deps: Vec<String>,
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
    )
    .await;

    match result {
        Ok(()) => {
            println!("{} Project updated successfully.", style("✔").green());
            Ok(())
        }
        Err(e) => match e {
            conduit_cli::core::error::CoreError::ProjectNotFound { slug } => {
                print_add_not_found_suggestions(api, &slug).await;
                Ok(())
            }
            _ => Err(Box::new(e)),
        },
    }
}

async fn print_add_not_found_suggestions(api: &ModrinthAPI, input: &str) {
    let query = input.split('@').next().unwrap_or(input);
    println!(
        "{} Project not found: {}",
        style("✘").red(),
        style(query).yellow().bold()
    );

    if let Ok(results) = api.search(query, 5, 0, "relevance", None).await
        && !results.hits.is_empty()
    {
        println!("{} Did you mean one of these?", style("?").yellow());
        for hit in results.hits {
            println!(
                "  {} {} ({})",
                style("-").dim(),
                style(hit.title).cyan(),
                style(hit.slug).dim()
            );
        }
    }
}
