use crate::config::ConduitConfig;
use crate::lock::ConduitLock;
use crate::modrinth::ModrinthAPI;
use console::style;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub async fn run(_api: &ModrinthAPI) -> Result<(), Box<dyn std::error::Error>> {
    let config_content =
        fs::read_to_string("conduit.json").map_err(|_| "❌ No conduit.json found.")?;
    let config: ConduitConfig = serde_json::from_str(&config_content)?;
    let lock = ConduitLock::load();

    let mods_dir = Path::new("mods");
    let mut files_on_disk = HashSet::new();
    if mods_dir.exists() {
        for entry in fs::read_dir(mods_dir)? {
            let entry = entry?;
            if let Some(name) = entry.file_name().to_str() {
                files_on_disk.insert(name.to_string());
            }
        }
    }

    println!("\n{}", style("Project Dependencies:").bold().underlined());

    let mut missing_mods = Vec::new();
    let mut tracked_files = HashSet::new();

    for (slug, version) in &config.mods {
        if let Some(locked) = lock.locked_mods.get(slug) {
            tracked_files.insert(locked.filename.clone());

            let status = if files_on_disk.contains(&locked.filename) {
                style("✔").green()
            } else {
                missing_mods.push(slug.clone());
                style("✘").red()
            };

            println!(
                "{} {} {}",
                status,
                style(slug).cyan(),
                style(format!("({})", version)).dim()
            );

            print_deps(slug, &lock, &mut tracked_files, &files_on_disk, 1);
        } else {
            missing_mods.push(slug.clone());
            println!(
                "{} {} {}",
                style("?").yellow(),
                style(slug).dim(),
                style("(not locked)").red().italic()
            );
        }
    }

    let orphans: Vec<_> = files_on_disk
        .iter()
        .filter(|f| !tracked_files.contains(*f))
        .collect();

    println!("\n{}", style("Summary:").dim());
    println!("  Total root mods: {}", config.mods.len());
    println!("  Files on disk:   {}", files_on_disk.len());

    if !missing_mods.is_empty() {
        println!(
            "\n{} Some mods are missing. Run {} to fix it.",
            style("⚠").red(),
            style("conduit install").yellow().bold()
        );
    }

    if !orphans.is_empty() {
        println!(
            "\n{} Found {} orphan files in /mods:",
            style("🗑").yellow(),
            orphans.len()
        );
        for orphan in orphans {
            println!("   - {}", style(orphan).dim().italic());
        }
        println!(
            "{}",
            style("   (Run 'conduit install' or 'conduit remove' to sync)").dim()
        );
    }

    Ok(())
}

fn print_deps(
    slug: &str,
    lock: &ConduitLock,
    tracked: &mut HashSet<String>,
    disk: &HashSet<String>,
    indent: usize,
) {
    if let Some(locked) = lock.locked_mods.get(slug) {
        for dep_id in &locked.dependencies {
            if let Some((dep_slug, dep_info)) =
                lock.locked_mods.iter().find(|(_, m)| &m.id == dep_id)
            {
                tracked.insert(dep_info.filename.clone());

                let pipe = if indent > 0 { "└──" } else { "" };
                let spacing = "    ".repeat(indent - 1);

                let on_disk = if disk.contains(&dep_info.filename) {
                    style("✔").green()
                } else {
                    style("✘").red()
                };

                println!(
                    "{}{} {} {} {}",
                    spacing,
                    style(pipe).dim(),
                    on_disk,
                    style(dep_slug).dim(),
                    style("(dep)").italic().dim()
                );

                print_deps(dep_slug, lock, tracked, disk, indent + 1);
            }
        }
    }
}
