use crate::ui::CliUi;
use console::style;
use conduit_cli::core::local_mods::add_local_mods_to_project;
use conduit_cli::core::installer::extra_deps::ExtraDepsPolicy;
use conduit_cli::core::installer::project::{add_mod_to_project, InstallProjectOptions};
use conduit_cli::core::paths::CorePaths;
use conduit_cli::core::error::CoreError;
use conduit_cli::modrinth::ModrinthAPI;
use reqwest::StatusCode;
use std::path::PathBuf;

pub async fn run(
    api: &ModrinthAPI,
    input: String,
    deps: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let paths = CorePaths::from_project_dir(".")?;

    if let Some(file_path) = parse_file_input(&input) {
        let mut jars = Vec::new();
        jars.push(file_path);
        for d in deps {
            jars.push(PathBuf::from(d));
        }

        let report = add_local_mods_to_project(&paths, jars)?;
        for added in report.added {
            println!(
                "{} Added local mod {}",
                style("✔").green(),
                style(added.filename).bold()
            );
        }
        return Ok(());
    }

    let mut ui = CliUi::new();

    let result = add_mod_to_project(
        api,
        &paths,
        &input,
        &mut ui,
        InstallProjectOptions {
            extra_deps_policy: ExtraDepsPolicy::Callback,
            strict: false,
            force: false,
        },
    )
    .await;

    match result {
        Ok(()) => Ok(()),
        Err(e) => {
            if is_project_not_found(&e) {
                print_add_not_found_suggestions(api, &input).await;
                return Ok(());
            }
            Err(e.into())
        }
    }
}

fn parse_file_input(input: &str) -> Option<PathBuf> {
    if let Some(rest) = input.strip_prefix("f:") {
        return Some(PathBuf::from(rest));
    }
    if let Some(rest) = input.strip_prefix("file:") {
        return Some(PathBuf::from(rest));
    }
    None
}

fn is_project_not_found(e: &CoreError) -> bool {
    match e {
        CoreError::Reqwest(re) => re.status() == Some(StatusCode::NOT_FOUND),
        _ => false,
    }
}

async fn print_add_not_found_suggestions(api: &ModrinthAPI, input: &str) {
    let query = input.split('@').next().unwrap_or(input);

    println!(
        "{} Project/slug not found: {}",
        style("✘").red(),
        style(query).yellow().bold()
    );
    println!(
        "{} Did you mean one of these?",
        style("?").yellow()
    );

    let results = match api.search(query, 5, 0, "relevance", None).await {
        Ok(r) => r,
        Err(_) => {
            println!("{} (Could not query Modrinth right now)", style("!").red());
            return;
        }
    };

    if results.hits.is_empty() {
        println!("{} No similar results found.", style("!").yellow());
        return;
    }

    for hit in results.hits {
        println!(
            "  {} {} ({})",
            style("-".to_string()).dim(),
            style(hit.title).cyan(),
            style(hit.slug).dim()
        );
    }
}
