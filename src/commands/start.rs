use crate::ui::CliUi;
use conduit_cli::core::{paths::CorePaths, start_server::start_server};
use std::error::Error;

pub async fn run(show_logs: bool, show_gui: bool) -> Result<(), Box<dyn Error>> {
    let paths = CorePaths::from_project_dir(".")?;
    let mut ui = CliUi::new();

    start_server(&paths, &mut ui, show_logs, show_gui).await
}
