#[derive(Debug, Clone)]
pub enum NotificationLevel { Info, Success, Warn, Error }

#[derive(Debug, Clone)]
pub enum SystemEvent {
    Log { level: NotificationLevel, message: String },
    TaskStarted(String),
    TaskFinished(String),
}
