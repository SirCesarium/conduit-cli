use conduit_cli::core::context::ConduitContext;
use conduit_cli::core::manager::ProjectManager;
use conduit_cli::core::manager::add::models::{
    AddRequest, ModSide, RemoteSource, ResourceSource, ResourceType,
};
use console::style;

pub async fn run(
    inputs: Vec<String>,
    _deps: Vec<String>,
    explicit_side: Option<ModSide>,
) -> Result<(), Box<dyn std::error::Error>> {
    let ctx = ConduitContext::load(".")?;
    let mut manager = ProjectManager::new(ctx);

    for input in inputs {
        let parts: Vec<&str> = input.split('@').collect();
        let slug = parts[0].to_string();
        let version = parts.get(1).map(std::string::ToString::to_string);

        let request = AddRequest {
            source: ResourceSource::Remote(RemoteSource::Modrinth { slug, version }),
            side: explicit_side.clone().unwrap_or(ModSide::Both),
            r#type: ResourceType::Mod,
            is_dependency: false,
        };

        if let Err(e) = manager.add_resource(request).await {
            eprintln!("{} Error installing {}: {}", style("✘").red(), input, e);
        }
    }

    println!("{} Project updated successfully.", style("✔").green());
    Ok(())
}
