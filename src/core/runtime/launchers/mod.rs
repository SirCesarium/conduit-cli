use std::path::Path;

use crate::core::{
    events::{CoreCallbacks, CoreEvent},
    io::server::config::ServerConfig,
    paths::CorePaths, runtime::launchers::generic_launcher::{LaunchCommand, launch_generic_server},
};

mod chat;
mod generic_launcher;
mod log_processor;

pub enum ServerLauncher {
    Neoforge,
    Vanilla,
}

impl ServerLauncher {
    pub fn is_ready(&self, paths: &CorePaths, version: &str) -> bool {
        match self {
            Self::Neoforge => {
                let dir = paths.neoforge_version_dir(version);
                dir.join("unix_args.txt").exists() && dir.join("win_args.txt").exists()
            }
            Self::Vanilla => paths.project_dir().join("server.jar").exists(),
        }
    }

    pub async fn launch(
        &self,
        paths: &CorePaths,
        config: &ServerConfig,
        loader_version: &str,
        show_logs: bool,
        show_gui: bool,
        callbacks: &mut dyn CoreCallbacks,
    ) {
        let launch_cmd = match self {
            Self::Neoforge => {
                let mut args = Vec::new();

                args.push(format!("-Xmx{}", config.performance.max_ram));
                args.push(format!("-Xms{}", config.performance.min_ram));

                for jvm_arg in &config.performance.jvm_args {
                    args.push(jvm_arg.clone());
                }

                let arg_file = if cfg!(target_os = "windows") {
                    "win_args.txt"
                } else {
                    "unix_args.txt"
                };

                let relative_args_path = Path::new("libraries")
                    .join("net/neoforged/neoforge")
                    .join(loader_version)
                    .join(arg_file);

                args.push(format!("@{}", relative_args_path.display()));

                if !show_gui {
                    args.push("nogui".to_string());
                }

                LaunchCommand {
                    program: "java".to_string(),
                    args,
                    current_dir: paths.project_dir().to_path_buf(),
                }
            }
            Self::Vanilla => LaunchCommand {
                program: "java".to_string(),
                args: vec![
                    format!("-Xmx{}", config.performance.max_ram),
                    format!("-Xms{}", config.performance.min_ram),
                    "-jar".into(),
                    "server.jar".into(),
                    "nogui".into(),
                ],
                current_dir: paths.project_dir().to_path_buf(),
            },
        };

        if let Err(e) = launch_generic_server(launch_cmd, callbacks, show_logs).await {
            callbacks.on_event(CoreEvent::Error(format!("Launcher failed: {e}")));
        }
    }
}
