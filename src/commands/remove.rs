use crate::config::ConduitConfig;
use crate::lock::ConduitLock;
use console::style;
use std::collections::HashSet;
use std::fs;

pub async fn run(input: String) -> Result<(), Box<dyn std::error::Error>> {
    let config_content =
        fs::read_to_string("conduit.json").map_err(|_| "❌ No conduit.json found.")?;
    let mut config: ConduitConfig = serde_json::from_str(&config_content)?;
    let mut lock = ConduitLock::load();

    if !config.mods.contains_key(&input) {
        println!(
            "{} Mod {} not found in config",
            style("!").yellow(),
            style(&input).bold()
        );
        return Ok(());
    }

    let mut dependents = Vec::new();
    if let Some(target_mod) = lock.locked_mods.get(&input) {
        let target_id = &target_mod.id;
        for (slug, info) in &lock.locked_mods {
            if slug != &input
                && config.mods.contains_key(slug)
                && info.dependencies.contains(target_id)
            {
                dependents.push(slug.clone());
            }
        }
    }

    if !dependents.is_empty() {
        println!(
            "{} Cannot remove {}: the following mods depend on it: {}",
            style("✘").red(),
            style(&input).bold(),
            style(dependents.join(", ")).yellow()
        );
        return Ok(());
    }

    println!(
        "\n{} {}",
        style("─── Removing").dim(),
        style(&input).red().bold()
    );

    config.mods.remove(&input);

    let mut mods_to_keep = HashSet::new();
    for slug in config.mods.keys() {
        collect_dependencies(slug, &lock, &mut mods_to_keep);
    }

    let all_locked_slugs: Vec<String> = lock.locked_mods.keys().cloned().collect();
    for slug in all_locked_slugs {
        if !mods_to_keep.contains(&slug)
            && let Some(mod_data) = lock.locked_mods.remove(&slug) {
                let dest_path = std::path::Path::new("mods").join(&mod_data.filename);
                if dest_path.exists() {
                    fs::remove_file(dest_path)?;
                }
                println!(
                    "{} Purged {}",
                    style("🗑").dim(),
                    style(&slug).dim().italic()
                );
            }
    }

    fs::write("conduit.json", serde_json::to_string_pretty(&config)?)?;
    lock.save()?;

    println!("{} Removed {}", style("✔").green(), style(&input).bold());
    Ok(())
}

fn collect_dependencies(slug: &str, lock: &ConduitLock, kept: &mut HashSet<String>) {
    if kept.contains(slug) {
        return;
    }
    if let Some(mod_data) = lock.locked_mods.get(slug) {
        kept.insert(slug.to_string());
        for dep_id in &mod_data.dependencies {
            if let Some((dep_slug, _)) = lock.locked_mods.iter().find(|(_, m)| &m.id == dep_id) {
                collect_dependencies(dep_slug, lock, kept);
            }
        }
    }
}
