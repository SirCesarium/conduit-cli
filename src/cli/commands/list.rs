use conduit_cli::core::io::project::ProjectFiles;
use conduit_cli::core::apis::modrinth::ModrinthAPI;
use conduit_cli::core::paths::CorePaths;
use conduit_cli::core::project::lister::{ListReport, build_list_report};
use console::style;

pub fn run(_api: &ModrinthAPI) -> Result<(), Box<dyn std::error::Error>> {
    let paths = CorePaths::from_project_dir(".")?;
    let config = ProjectFiles::load_manifest(&paths)?;
    let lock = ProjectFiles::load_lock(&paths)?;
    let report = build_list_report(&paths, &config, &lock)?;

    println!("\n{}", style("Project Dependencies:").bold().underlined());

    for (slug, version) in &config.mods {
        if let Some(locked) = lock.locked_mods.get(slug) {
            let status = if report.files_on_disk.contains(&locked.filename) {
                style("✔").green()
            } else {
                style("✘").red()
            };

            println!(
                "{} {} {}",
                status,
                style(slug).cyan(),
                style(format!("({version})")).dim()
            );

            print_deps(slug, &report, 1);
        } else {
            println!(
                "{} {} {}",
                style("?").yellow(),
                style(slug).dim(),
                style("(not locked)").red().italic()
            );
        }
    }

    println!("\n{}", style("Summary:").dim());
    println!("  Total root mods: {}", config.mods.len());
    println!("  Files on disk:   {}", report.files_on_disk.len());

    if !report.missing_root_slugs.is_empty() {
        println!(
            "\n{} Some mods are missing. Run {} to fix it.",
            style("⚠").red(),
            style("conduit install").yellow().bold()
        );
    }

    if !report.orphan_files.is_empty() {
        println!(
            "\n{} Found {} orphan files in /mods:",
            style("🗑").yellow(),
            report.orphan_files.len()
        );
        for orphan in &report.orphan_files {
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
    report: &ListReport,
    indent: usize,
) {
    if let Some(deps) = report.dependency_map.get(slug) {
        for (dep_slug, dep_info) in deps {
            let pipe = if indent > 0 { "└──" } else { "" };
            let spacing = "    ".repeat(indent - 1);

            let on_disk = if report.files_on_disk.contains(&dep_info.filename) {
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

            print_deps(dep_slug, report, indent + 1);
        }
    }
}
