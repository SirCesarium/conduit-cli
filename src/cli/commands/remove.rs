use conduit_cli::core::events::{CoreCallbacks, CoreEvent};
use conduit_cli::core::io::project::ProjectFiles;
use conduit_cli::core::mods::remover::remove_mod;
use conduit_cli::core::paths::CorePaths;
use console::style;

pub fn run(input: &str) -> Result<(), Box<dyn std::error::Error>> {
    let paths = CorePaths::from_project_dir(".")?;
    let mut config = ProjectFiles::load_manifest(&paths)?;
    let mut lock = ProjectFiles::load_lock(&paths)?;

    if !config.mods.contains_key(input) {
        println!(
            "{} Mod {} not found in config",
            style("!").yellow(),
            style(&input).bold()
        );
        return Ok(());
    }

    let mut dependents = Vec::new();
    if let Some(target_mod) = lock.locked_mods.get(input) {
        let target_id = &target_mod.id;
        for (slug, info) in &lock.locked_mods {
            if slug != input
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

    let mut cb = CliCallbacks;
    let _ = remove_mod(&paths, input, &mut config, &mut lock, &mut cb)?;

    ProjectFiles::save_manifest(&paths, &config)?;
    ProjectFiles::save_lock(&paths, &lock)?;

    println!("{} Removed {}", style("✔").green(), style(&input).bold());
    Ok(())
}

struct CliCallbacks;

impl CoreCallbacks for CliCallbacks {
    fn on_event(&mut self, event: CoreEvent) {
        if let CoreEvent::Purged { slug } = event {
            println!("{} Purged {}", style("🗑").dim(), style(slug).dim().italic());
        }
    }
}
