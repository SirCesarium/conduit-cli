use conduit_cli::core::ui::ConduitUI;
use conduit_cli::core::ui::package::PackageEvent;
use conduit_cli::core::ui::system::SystemEvent;
use conduit_cli::core::events::{CoreCallbacks, CoreEvent, DownloadProgress};
use console::Term;
use indicatif::ProgressBar;

#[allow(unused)]
pub struct CliUi {
    term: Term,
    download_pb: Option<ProgressBar>,
    spinner_pb: Option<ProgressBar>,
    download_filename: Option<String>,
    download_total: Option<u64>,
}

impl CliUi {
    pub fn new() -> Self {
        Self {
            term: Term::stdout(),
            download_pb: None,
            spinner_pb: None,
            download_filename: None,
            download_total: None,
        }
    }
}

impl CoreCallbacks for CliUi {
    fn on_event(&mut self, _event: CoreEvent) {}
    fn on_download_progress(&mut self, _progress: DownloadProgress) {}
}

impl ConduitUI for CliUi {
    fn handle_package(&self, _event: PackageEvent) {}
    fn handle_system(&self, _event: SystemEvent) {}
}