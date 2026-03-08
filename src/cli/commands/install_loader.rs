use crate::cli::ui::CliUi;
use conduit_cli::core::error::CoreResult;
use conduit_cli::core::paths::CorePaths;
use conduit_cli::core::server::install::install_loader;

pub async fn run() -> CoreResult<()> {
    let paths = CorePaths::from_project_dir(".")?;
    let mut ui = CliUi::new();

    install_loader(&paths, &mut ui).await
}
