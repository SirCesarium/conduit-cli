use crate::config::ConduitConfig;
use console::style;
use std::fs;

pub fn run(
    name: Option<String>,
    loader: Option<String>,
    yes: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = ConduitConfig::default();

    if yes {
        if let Some(n) = name {
            config.name = n;
        }
        if let Some(l) = loader {
            config.loader = l;
        }
    } else {
        config.name = inquire::Text::new("Project name:")
            .with_default(&config.name)
            .prompt()?;

        config.mc_version = inquire::Text::new("Minecraft version:")
            .with_default(&config.mc_version)
            .prompt()?;

        config.loader = inquire::Text::new("Loader (e.g. neoforge@219):")
            .with_default(&config.loader)
            .prompt()?;
    }

    let json = serde_json::to_string_pretty(&config)?;
    fs::write("conduit.json", json)?;

    println!(
        "{} Created {} for project {}!",
        style("⚡").yellow(),
        style("conduit.json").cyan(),
        style(&config.name).bold().green()
    );

    Ok(())
}
