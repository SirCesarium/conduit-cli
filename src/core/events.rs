#[derive(Debug, Clone)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Chat,
    Command,
}

#[derive(Debug, Clone)]
pub enum CoreEvent {
    Info(String),
    Success(String),
    Warning(String),
    Error(String),
    Installed { slug: String, title: String },
    AddedAsDependency { slug: String },
    AlreadyInstalled { slug: String },
    LinkedFile { filename: String },
    Purged { slug: String },
    ChatModeStarted { sender: String },
    ChatModeStopped,
    ChatMessageSent { sender: String, message: String },
    ChatPromptRequested { sender: String },
    ServerLogEvent { level: LogLevel, message: String, timestamp: String },
    ServerStopEvent(String),
    WorldPreparationStarted,
    WorldPreparationProgress { percentage: u8 },
    WorldPreparationFinished,
    StartingServer,
    TaskStarted(String),
    TaskFinished,
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
