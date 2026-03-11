use crate::cli::commands::Cmds;
use crate::cli::ui::UI;
use clap::Args;
use conduit_cli::core::domain::loader::Loader;
use miette::IntoDiagnostic;

#[derive(Args)]
pub struct InstallArgs {
    #[arg(short, long)]
    pub loader: Option<String>,
}

impl Cmds {
    pub async fn install(&self, args: InstallArgs) -> miette::Result<()> {
        let loader = if let Some(raw) = args.loader {
            let (name, version) = match raw.split_once('@') {
                Some((n, v)) => (n, Some(v)),
                None => (raw.as_str(), None),
            };

            Loader::from_string(name, version).map_err(|e| miette::miette!(e.to_string()))?
        } else {
            let manifest = self.ctx.manifest.read().await;
            manifest.project.loader.clone()
        };

        UI::info(format!("Installing loader: {loader}"));

        self.pj_manager
            .install_loader(loader.clone())
            .await
            .into_diagnostic()?;

        UI::info(format!("Successfully installed {loader}"));

        Ok(())
    }
}
