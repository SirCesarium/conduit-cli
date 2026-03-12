use crate::cli::{commands::Cmds, ui::UI};
use clap::Args;
use conduit_cli::core::domain::addon::AddonType;
use miette::IntoDiagnostic;

#[derive(Args)]
pub struct AddArgs {
    #[command(subcommand)]
    pub target: Option<AddTarget>,

    #[arg(num_args = 1..)]
    pub slugs: Vec<String>,

    #[arg(short, long, num_args = 1..)]
    pub deps: Vec<std::path::PathBuf>,
}

#[derive(clap::Subcommand)]
pub enum AddTarget {
    Mod { slugs: Vec<String> },
    Plugin { slugs: Vec<String> },
    Datapack { slugs: Vec<String> },
}

impl Cmds {
    pub async fn add(&self, args: AddArgs) -> miette::Result<()> {
        let (slugs, target_type) = if let Some(target) = args.target {
            match target {
                AddTarget::Mod { slugs } => (slugs, AddonType::Mod),
                AddTarget::Plugin { slugs } => (slugs, AddonType::Plugin),
                AddTarget::Datapack { slugs } => (slugs, AddonType::Datapack),
            }
        } else {
            if args.slugs.is_empty() {
                UI::error(
                    "No slugs provided. Usage: conduit add <slugs> or conduit add mod <slugs>",
                );
                return Err(miette::miette!("Unable to infer target type or slugs"));
            }
            (args.slugs.clone(), AddonType::Mod)
        };

        UI::logo();

        UI::info(format!("Resolving {} dependencies...", slugs.len()));

        self.pj_manager
            .add_addons(slugs.clone(), target_type.clone())
            .await
            .into_diagnostic()?;

        for slug in &slugs {
            UI::action("Installed", format!("{slug} ({target_type:?})"));
        }

        UI::success(format!("Added {} addons to the project", slugs.len()));

        Ok(())
    }
}
