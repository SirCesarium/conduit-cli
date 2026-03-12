use crate::cli::{commands::Cmds, ui::UI};
use clap::Args;
use conduit_cli::errors::ConduitError;
use console::style;
use inquire::Confirm;
use miette::IntoDiagnostic;

#[derive(Args)]
pub struct StartArgs {
    #[arg(short, long)]
    tui: bool,
}

impl Cmds {
    pub async fn start(&self, args: StartArgs) -> miette::Result<()> {
        if args.tui {
            UI::warn("Not implemented yet!");
        }

        let result = self.pj_manager.start().await;

        if let Err(e) = result {
            match e {
                ConduitError::NotInstalled => {
                    UI::warn("Loader not found or not fully installed.");

                    let (loader_to_install, loader_name) = {
                        let manifest = self.pj_manager.ctx.manifest.read().await;
                        (
                            manifest.project.loader.clone(),
                            manifest.project.loader.pretty_name(),
                        )
                    };

                    let ans = Confirm::new(&format!(
                        "Do you want to install the {} loader now?",
                        style(loader_name).magenta()
                    ))
                    .with_default(true)
                    .prompt()
                    .into_diagnostic()?;

                    if ans {
                        UI::info("Installing missing loader...");
                        self.pj_manager
                            .install_loader(loader_to_install)
                            .await
                            .into_diagnostic()?;
                        UI::success("Loader installed successfully!");

                        UI::info("Starting server...");
                        self.pj_manager.start().await.into_diagnostic()?;
                    } else {
                        return Err(miette::miette!("Server cannot start without the loader."));
                    }
                }
                _ => return Err(e).into_diagnostic(),
            }
        }

        Ok(())
    }
}
