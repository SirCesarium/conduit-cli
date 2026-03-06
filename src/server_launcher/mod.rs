use std::path::PathBuf;

use crate::{
    core::events::{CoreCallbacks, CoreEvent},
    server_launcher::generic_launcher::{LaunchCommand, launch_generic_server},
};

mod chat;
mod generic_launcher;
mod log_processor;

pub enum ServerLauncher {
    Neoforge,
    Vanilla,
}

impl ServerLauncher {
    pub async fn launch(
        &self,
        path: PathBuf,
        show_logs: bool,
        show_gui: bool,
        callbacks: &mut dyn CoreCallbacks,
    ) {
        let launch_cmd = match self {
            Self::Neoforge => {
                let (shell, mut args) = if cfg!(target_os = "windows") {
                    ("cmd", vec!["/C", "run.bat"])
                } else {
                    ("sh", vec!["run.sh"])
                };
                if !show_gui {
                    args.push("nogui");
                }

                LaunchCommand {
                    program: shell.to_string(),
                    args: args.iter().map(|s| s.to_string()).collect(),
                    current_dir: path,
                }
            }
            Self::Vanilla => LaunchCommand {
                program: "java".to_string(),
                args: vec![
                    "-Xmx2G".into(),
                    "-jar".into(),
                    "server.jar".into(),
                    "nogui".into(),
                ],
                current_dir: path,
            },
        };

        if let Err(e) = launch_generic_server(launch_cmd, callbacks, show_logs).await {
            callbacks.on_event(CoreEvent::Error(format!("Launcher failed: {}", e)));
        }
    }
}
