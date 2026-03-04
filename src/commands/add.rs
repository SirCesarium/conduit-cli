use crate::config::ConduitConfig;
use crate::inspector::JarInspector;
use crate::lock::{ConduitLock, LockedMod};
use crate::modrinth::ModrinthAPI;
use crate::progress::ConduitProgress;
use async_recursion::async_recursion;
use console::{Term, style};
use futures_util::StreamExt;
use inquire::Select;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub async fn run(api: &ModrinthAPI, input: String) -> Result<(), Box<dyn std::error::Error>> {
    let config_content = fs::read_to_string("conduit.json")
        .map_err(|_| "❌ No conduit.json found. Run 'conduit init' first.")?;
    let mut config: ConduitConfig = serde_json::from_str(&config_content)?;
    let mut lock = ConduitLock::load();

    install_recursive(api, &input, &mut config, &mut lock, true).await?;

    fs::write("conduit.json", serde_json::to_string_pretty(&config)?)?;
    lock.save()?;
    Ok(())
}

#[async_recursion]
pub async fn install_recursive(
    api: &ModrinthAPI,
    input: &str,
    config: &mut ConduitConfig,
    lock: &mut ConduitLock,
    is_root: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let parts: Vec<&str> = input.split('@').collect();
    let slug_or_id = parts[0];

    let project = api.get_project(slug_or_id).await?;
    let current_slug = project.slug;

    if is_root && config.mods.contains_key(&current_slug) {
        println!(
            "{} Mod {} is already installed",
            style("ℹ").cyan(),
            style(&current_slug).bold()
        );
        return Ok(());
    }

    if !is_root && lock.locked_mods.contains_key(&current_slug) {
        return Ok(());
    }

    if is_root && lock.locked_mods.contains_key(&current_slug) {
        config
            .mods
            .insert(current_slug.clone(), "latest".to_string());
        println!(
            "{} Added {} as dependency",
            style("✔").green(),
            style(&current_slug).bold()
        );
        return Ok(());
    }

    println!(
        "\n{} {}",
        style("─── Installing").dim(),
        style(&project.title).magenta().bold()
    );

    let loader_filter = config.loader.split('@').next().unwrap_or("fabric");

    let versions = api
        .get_versions(&current_slug, Some(loader_filter), Some(&config.mc_version))
        .await?;

    let selected_version = match versions.first() {
        Some(v) => v,
        None => return Err(format!("No compatible version for {}", current_slug).into()),
    };

    let file = selected_version
        .files
        .iter()
        .find(|f| f.primary)
        .or(selected_version.files.first())
        .ok_or_else(|| {
            format!(
                "No files available for version {}",
                selected_version.version_number
            )
        })?;

    let sha1 = file.hashes.get("sha1").cloned().unwrap_or_default();

    let cache_dir = dirs::data_local_dir()
        .ok_or("❌ Could not find local data directory")?
        .join("conduit")
        .join("cache");
    fs::create_dir_all(&cache_dir)?;
    let cached_path = cache_dir.join(format!("{}.jar", sha1));

    let mods_dir = Path::new("mods");
    fs::create_dir_all(mods_dir)?;
    let dest_path = mods_dir.join(&file.filename);

    if !cached_path.exists() {
        let response = reqwest::get(&file.url).await?;
        let pb = ConduitProgress::download_style(response.content_length().unwrap_or(file.size));
        pb.set_message(format!(
            "{} {}",
            style("").cyan(),
            style(&file.filename).dim()
        ));

        let mut cache_file = fs::File::create(&cached_path)?;
        let mut stream = response.bytes_stream();
        while let Some(item) = stream.next().await {
            let chunk = item?;
            cache_file.write_all(&chunk)?;
            pb.inc(chunk.len() as u64);
        }
        pb.finish_and_clear();
    }

    if dest_path.exists() {
        fs::remove_file(&dest_path)?;
    }
    fs::hard_link(&cached_path, &dest_path)?;

    if is_root {
        config.mods.insert(
            current_slug.clone(),
            selected_version.version_number.clone(),
        );
    }

    let mut current_deps = Vec::new();
    for dep in &selected_version.dependencies {
        if dep.dependency_type == "required"
            && let Some(proj_id) = &dep.project_id
        {
            current_deps.push(proj_id.clone());
        }
    }

    lock.locked_mods.insert(
        current_slug.clone(),
        LockedMod {
            id: selected_version.project_id.clone(),
            version_id: selected_version.id.clone(),
            filename: file.filename.clone(),
            url: file.url.clone(),
            hash: sha1,
            dependencies: current_deps.clone(),
        },
    );

    for dep_id in current_deps {
        install_recursive(api, &dep_id, config, lock, false).await?;
    }

    crawl_extra_dependencies(api, &dest_path, config, lock, &current_slug).await?;

    println!(
        "{} Installed {}",
        style("✔").green(),
        style(&project.title).bold()
    );
    Ok(())
}

async fn crawl_extra_dependencies(
    api: &ModrinthAPI,
    jar_path: &PathBuf,
    config: &mut ConduitConfig,
    lock: &mut ConduitLock,
    parent_slug: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let internal_deps = match JarInspector::inspect_neoforge(jar_path) {
        Ok(deps) => deps,
        Err(_) => return Ok(()),
    };

    let loader_filter = config
        .loader
        .split('@')
        .next()
        .unwrap_or("neoforge")
        .to_string();
    let mc_version = config.mc_version.clone();

    for tech_id in internal_deps {
        let is_installed = lock.locked_mods.values().any(|m| m.id == tech_id)
            || lock.locked_mods.contains_key(&tech_id)
            || config.mods.contains_key(&tech_id);

        if is_installed {
            continue;
        }

        let facets = format!(
            "[[\"categories:{}\"],[\"versions:{}\"]]",
            loader_filter, mc_version
        );
        let search_results = api
            .search(&tech_id, 5, 0, "relevance", Some(facets))
            .await?;

        let mut options: Vec<String> = Vec::new();
        options.push(style("X Skip dependency").red().to_string());

        let mut exact_match_slug = None;
        if let Ok(exact) = api.get_project(&tech_id).await {
            exact_match_slug = Some(exact.slug.clone());
            options.push(format!(
                "{} {} ({})",
                style("!").yellow(),
                exact.title,
                exact.slug
            ));
        }

        for hit in &search_results.hits {
            if Some(&hit.slug) != exact_match_slug.as_ref() {
                options.push(format!("{} ({})", hit.title, hit.slug));
            }
        }

        if options.len() <= 1 {
            continue;
        }

        let term = Term::stdout();
        let file_name = jar_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown file");
        let prompt = format!(
            "Dependency {} needed for {}:",
            style(&tech_id).bold().yellow(),
            style(file_name).dim()
        );

        let selection = Select::new(&prompt, options).with_page_size(7).prompt();

        term.clear_last_lines(1)?;

        match selection {
            Ok(choice) if !choice.contains("Skip dependency") => {
                let slug_to_install = if choice.contains("!") {
                    exact_match_slug.clone().unwrap_or(tech_id.clone())
                } else {
                    search_results
                        .hits
                        .iter()
                        .find(|hit| choice.contains(&hit.slug))
                        .map(|hit| hit.slug.clone())
                        .unwrap_or(tech_id.clone())
                };

                install_recursive(api, &slug_to_install, config, lock, false).await?;

                if let Some(installed_mod) = lock.locked_mods.get(&slug_to_install) {
                    let installed_id = installed_mod.id.clone();
                    if let Some(parent) = lock.locked_mods.get_mut(parent_slug)
                        && !parent.dependencies.contains(&installed_id)
                    {
                        parent.dependencies.push(installed_id);
                    }
                }
            }
            _ => (),
        }
    }

    Ok(())
}
