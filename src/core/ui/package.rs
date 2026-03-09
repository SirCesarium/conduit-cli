#[derive(Debug, Clone)]
pub enum PackageEvent {
    DownloadStarted { id: String, name: String, total: Option<u64> },
    DownloadProgress { id: String, current: u64 },
    DownloadFinished { id: String },
    Resolving { name: String },
}
