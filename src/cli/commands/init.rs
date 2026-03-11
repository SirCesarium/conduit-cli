use crate::cli::{commands::Cmds, errors::CliError, ui::UI};
use clap::{Args, ValueEnum};
use conduit_cli::{core::domain::loader::Loader, errors::ConduitError};
use inquire::{Select, Text};
use strum::Display;

#[derive(Args)]
pub struct InitArgs {
    #[arg(short, long)]
    pub yes: bool,

    #[arg(short, long)]
    pub name: Option<String>,

    #[arg(alias = "mc", long, default_value = "1.21.1")]
    pub minecraft: String,

    #[arg(short, long, default_value = "vanilla")]
    pub loader: CliLoader,

    #[arg(long)]
    pub loader_version: Option<String>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Display)]
pub enum CliLoader {
    Vanilla,
    Fabric,
    Forge,
    Neoforge,
    Paper,
    Purpur,
}

impl Cmds {
    pub async fn init(&self, args: InitArgs) -> miette::Result<()> {
        if self.ctx.paths.manifest().exists() {
            return Err(CliError::from(ConduitError::AlreadyInitialized(
                "conduit.toml exists".into(),
            ))
            .into());
        }

        UI::logo();

        let mut project_name = args.name.clone().unwrap_or_else(|| {
            self.ctx
                .paths
                .root
                .file_name()
                .and_then(|name| name.to_str())
                .map_or_else(
                    || "conduit-project".to_string(),
                    std::string::ToString::to_string,
                )
        });

        let mut mc_version = args.minecraft.clone();
        let mut selected_loader = args.loader;
        let mut loader_version = args.loader_version.clone();

        if !args.yes {
            let config = UI::render_config();

            project_name = Text::new("Project name:")
                .with_render_config(config)
                .with_default(&project_name)
                .prompt()
                .map_err(|_| ConduitError::Validation("Prompt cancelled".into()))
                .map_err(CliError::from)?;

            mc_version = Text::new("Minecraft version:")
                .with_render_config(config)
                .with_default(&mc_version)
                .prompt()
                .map_err(|_| ConduitError::Validation("Prompt cancelled".into()))
                .map_err(CliError::from)?;

            let loader_options = vec![
                CliLoader::Vanilla,
                CliLoader::Fabric,
                CliLoader::Forge,
                CliLoader::Neoforge,
                CliLoader::Paper,
                CliLoader::Purpur,
            ];

            selected_loader = Select::new("Select loader:", loader_options)
                .with_render_config(config)
                .prompt()
                .map_err(|_| ConduitError::Validation("Prompt cancelled".into()))
                .map_err(CliError::from)?;

            if matches!(selected_loader, CliLoader::Forge | CliLoader::Neoforge) {
                let default_ver = loader_version.as_deref().unwrap_or("21.1.219");
                loader_version = Some(
                    Text::new("Loader version:")
                        .with_render_config(config)
                        .with_default(default_ver)
                        .prompt()
                        .map_err(|_| ConduitError::Validation("Prompt cancelled".into()))
                        .map_err(CliError::from)?,
                );
            }
        }

        if loader_version.is_some()
            && !matches!(selected_loader, CliLoader::Forge | CliLoader::Neoforge)
        {
            return Err(CliError::from(ConduitError::Validation(format!(
                "Loader '{selected_loader}' does not support custom versions."
            )))
            .into());
        }

        let domain_loader = match selected_loader {
            CliLoader::Vanilla => Loader::Vanilla,
            CliLoader::Fabric => Loader::Fabric,
            CliLoader::Paper => Loader::Paper,
            CliLoader::Purpur => Loader::Purpur,
            CliLoader::Forge => {
                let version = loader_version
                    .filter(|v| !v.is_empty())
                    .ok_or_else(|| {
                        ConduitError::Validation("Forge requires a loader version".to_string())
                    })
                    .map_err(CliError::from)?;
                Loader::Forge { version }
            }
            CliLoader::Neoforge => {
                let version = loader_version
                    .filter(|v| !v.is_empty())
                    .ok_or_else(|| {
                        ConduitError::Validation("Neoforge requires a loader version".to_string())
                    })
                    .map_err(CliError::from)?;
                Loader::Neoforge { version }
            }
        };

        self.pj_manager
            .init(project_name.clone(), mc_version, domain_loader)
            .await
            .map_err(CliError::from)?;

        UI::success(format!("Project '{project_name}' ready"));

        Ok(())
    }
}
