#[derive(Debug, Clone)]
pub enum CoreEvent {
    Info(String),
    Warning(String),
    Installed { slug: String, title: String },
    AddedAsDependency { slug: String },
    AlreadyInstalled { slug: String },
    LinkedFile { filename: String },
    Purged { slug: String },
}

#[derive(Debug, Clone)]
pub struct DownloadProgress {
    pub bytes_downloaded: u64,
    pub total_bytes: Option<u64>,
    pub filename: String,
}

pub trait CoreCallbacks {
    fn on_event(&mut self, _event: CoreEvent) {}
    fn on_download_progress(&mut self, _progress: DownloadProgress) {}
}
