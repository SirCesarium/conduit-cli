use conduit_cli::core::initializer::{init_project, InitParams};
use conduit_cli::core::paths::CorePaths;
use console::style;

pub fn run(
    name: Option<String>,
    loader: Option<String>,
    yes: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let paths = CorePaths::from_project_dir(".")?;
    let mut params = InitParams {
        name,
        mc_version: None,
        loader,
    };

    if yes {
        let config = init_project(&paths, params)?;

        println!(
            "{} Created {} for project {}!",
            style("⚡").yellow(),
            style("conduit.json").cyan(),
            style(&config.name).bold().green()
        );
        return Ok(());
    } else {
        let default_name = "conduit-server".to_string();
        params.name = Some(
            inquire::Text::new("Project name:")
                .with_default(params.name.as_deref().unwrap_or(&default_name))
                .prompt()?,
        );

        let default_mc = "1.21.1".to_string();
        params.mc_version = Some(
            inquire::Text::new("Minecraft version:")
                .with_default(params.mc_version.as_deref().unwrap_or(&default_mc))
                .prompt()?,
        );

        let default_loader = "neoforge@latest".to_string();
        params.loader = Some(
            inquire::Text::new("Loader (e.g. neoforge@219):")
                .with_default(params.loader.as_deref().unwrap_or(&default_loader))
                .prompt()?,
        );
    }

    let config = init_project(&paths, params)?;

    println!(
        "{} Created {} for project {}!",
        style("⚡").yellow(),
        style("conduit.json").cyan(),
        style(&config.name).bold().green()
    );

    Ok(())
}
