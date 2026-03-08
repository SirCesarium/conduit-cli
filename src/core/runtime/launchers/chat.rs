use crate::core::events::{CoreCallbacks, CoreEvent};
use serde::Serialize;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::mpsc::Sender;

#[derive(Serialize)]
struct RawText {
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bold: Option<bool>,
}

impl RawText {
    fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            color: None,
            bold: None,
        }
    }

    fn color(mut self, color: &str) -> Self {
        self.color = Some(color.to_string());
        self
    }

    fn bold(mut self, is_bold: bool) -> Self {
        self.bold = Some(is_bold);
        self
    }
}
pub struct ChatManager {
    active: Arc<AtomicBool>,
    name: Arc<Mutex<String>>,
}

impl ChatManager {
    pub fn new() -> Self {
        Self {
            active: Arc::new(AtomicBool::new(false)),
            name: Arc::new(Mutex::new("Server".to_string())),
        }
    }

    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }

    pub fn get_name(&self) -> String {
        self.name.lock().unwrap().clone()
    }

    pub async fn handle_input(
        &self,
        input: &str,
        tx_server: &Sender<String>,
        callbacks: &mut dyn CoreCallbacks,
    ) -> bool {
        let input = input.trim();

        if input.is_empty() {
            if self.is_active() {
                callbacks.on_event(CoreEvent::ChatPromptRequested {
                    sender: self.get_name(),
                });
                return true;
            }
            return false;
        }

        if input == ":chat" || input.starts_with(":chat ") {
            let name = if input.starts_with(":chat ") {
                input.replace(":chat ", "").trim().to_string()
            } else {
                "Server".to_string()
            };

            self.active.store(true, Ordering::SeqCst);
            if let Ok(mut n) = self.name.lock() {
                n.clone_from(&name);
            }

            callbacks.on_event(CoreEvent::ChatModeStarted { sender: name });
            return true;
        }

        if input == ":exit" || input == ":e" || input == ":q" {
            if self.is_active() {
                self.active.store(false, Ordering::SeqCst);
                callbacks.on_event(CoreEvent::ChatModeStopped);
                return true;
            }
            return false;
        }

        if self.is_active() {
            let name = self.get_name();

            let components = vec![
                RawText::new(&format!("<{name}> ")).color("gold").bold(true),
                RawText::new(input).color("white").bold(false)
            ];

            if let Ok(json_data) = serde_json::to_string(&components) {
                let command = format!("tellraw @a {json_data}");
                let _ = tx_server.send(command).await;
            }

            return true;
        }

        false
    }

    pub fn deactivate(&self) {
        self.active.store(false, Ordering::SeqCst);
    }
}
