use crate::core::events::{CoreCallbacks, CoreEvent, LogLevel};

pub fn parse_log(line: &str) -> (LogLevel, String, String) {
    let timestamp = if line.starts_with('[') && line.len() > 10 {
        line.get(1..9).unwrap_or("        ").to_string()
    } else {
        "        ".to_string()
    };

    let raw_msg = line.split("]: ").last().unwrap_or(line);
    let clean_msg = raw_msg.replace("[Not Secure] ", "").trim().to_string();

    let level = if line.contains("/INFO]") {
        if clean_msg.starts_with('<') && clean_msg.contains('>') {
            LogLevel::Chat
        } else {
            LogLevel::Info
        }
    } else if line.contains("/WARN]") {
        LogLevel::Warning
    } else if line.contains("/ERROR]") {
        LogLevel::Error
    } else {
        LogLevel::Info
    };

    (level, clean_msg, timestamp)
}

pub fn notify_startup(started: &mut bool, show_logs: &mut bool, callbacks: &mut dyn CoreCallbacks) {
    if !*started {
        *started = true;
        *show_logs = true;
        callbacks.on_event(CoreEvent::Success("Server is online!".into()));
    }
}

pub fn handle_exit_failure(callbacks: &mut dyn CoreCallbacks, buffer: Vec<String>) {
    callbacks.on_event(CoreEvent::WorldPreparationFinished);
    callbacks.on_event(CoreEvent::Error("SERVER CRASHED!".into()));

    for line in buffer {
        let (level, message, timestamp) = parse_log(&line);
        callbacks.on_event(CoreEvent::ServerLogEvent {
            level,
            message,
            timestamp,
        });
    }
}
