use console::style;
use conduit_cli::core::events::{CoreCallbacks, CoreEvent};
use conduit_cli::core::io::{load_config, load_lock, save_config, save_lock};
use conduit_cli::core::paths::CorePaths;
use conduit_cli::core::remover::remove_mod;

pub async fn run(input: String) -> Result<(), Box<dyn std::error::Error>> {
    let paths = CorePaths::from_project_dir(".")?;
    let mut config = load_config(&paths)?;
    let mut lock = load_lock(&paths)?;

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

    let mut cb = CliCallbacks;
    let _ = remove_mod(&paths, &input, &mut config, &mut lock, &mut cb)?;

    save_config(&paths, &config)?;
    save_lock(&paths, &lock)?;

    println!("{} Removed {}", style("✔").green(), style(&input).bold());
    Ok(())
}

struct CliCallbacks;

impl CoreCallbacks for CliCallbacks {
    fn on_event(&mut self, event: CoreEvent) {
        match event {
            CoreEvent::Purged { slug } => {
                println!("{} Purged {}", style("🗑").dim(), style(slug).dim().italic());
            }
            _ => {}
        }
    }
}
