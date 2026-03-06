use conduit_cli::core::filesystem::config::InstanceType;
use conduit_cli::core::initializer::{InitParams, init_project};
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
        instance_type: None,
        mc_version: None,
        loader,
    };

    if yes {
        let config = init_project(&paths, params)?;

        println!(
            "{} Created {} for project {}!",
            style("⚡").magenta(),
            style("conduit.json").cyan(),
            style(&config.name).bold().green()
        );
        return Ok(());
    } else {
        let current_dir = std::env::current_dir()?;

        let default_name = current_dir
            .file_name()
            .and_then(|os_str| os_str.to_str())
            .map(|s| s.to_lowercase().replace(' ', "-"))
            .unwrap_or_else(|| "conduit-server".to_string());

        params.name = Some(
            inquire::Text::new("Project name:")
                .with_default(params.name.as_deref().unwrap_or(&default_name))
                .prompt()?,
        );

        let options = vec![InstanceType::Server, InstanceType::Client];
        params.instance_type = Some(
            inquire::Select::new("Instance type:", options)
                .with_starting_cursor(0)
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
            inquire::Text::new("Loader (e.g. neoforge@21.1.219):")
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
