use std::path::PathBuf;

use crate::cli::commands::Cmds;
use clap::{Args, ValueEnum};
use conduit_cli::core::engine::manager::export::ModpackType;
use miette::IntoDiagnostic;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ModpackTypeCli {
    Conduit,
    Mrpack,
}

#[derive(Args)]
pub struct ExportArgs {
    pub out: PathBuf,

    #[arg(short, long, value_enum)]
    pub r#type: ModpackTypeCli,
}

impl Cmds {
    pub async fn export(&self, args: ExportArgs) -> miette::Result<()> {
        let r#type = match args.r#type {
            ModpackTypeCli::Conduit => ModpackType::Conduit,
            ModpackTypeCli::Mrpack => unimplemented!(),
        };

        let _ = self
            .pj_manager
            .export(r#type, args.out)
            .await
            .into_diagnostic()?;

        Ok(())
    }
}
