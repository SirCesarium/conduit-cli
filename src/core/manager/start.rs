use crate::core::io::TomlFile;
use crate::core::manager::ProjectManager;
use crate::domain::loader::Loader;
use crate::paths::ConduitPaths;
use crate::schemas::lock::Lockfile;
use std::process::Stdio;
use thiserror::Error;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

#[derive(Error, Debug)]
pub enum StartError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("lockfile not found, please install first")]
    NotInstalled,
    #[error("could not find a valid entry point (server.jar or args file)")]
    NoEntryPoint,
}

impl ProjectManager {
    pub async fn start(&self) -> Result<(), StartError> {
        let lock_path = ConduitPaths::get_lock_path(&self.project_root);
        let lock = Lockfile::load(lock_path)
            .await
            .map_err(|_| StartError::NotInstalled)?;

        let mut cmd = Command::new("java");
        cmd.current_dir(&self.project_root);
        cmd.kill_on_drop(true);

        self.setup_loader_args(&mut cmd, &lock).await?;
        cmd.arg("nogui");

        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::inherit());

        let mut child = cmd.spawn()?;
        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();

        let stdin_task = tokio::spawn(Self::bridge_stdin(stdin));

        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            println!("{line}");
        }

        let _ = child.wait().await;
        stdin_task.abort();

        Ok(())
    }

    async fn setup_loader_args(
        &self,
        cmd: &mut Command,
        lock: &Lockfile,
    ) -> Result<(), StartError> {
        match &lock.instance.loader {
            Loader::Neoforge { version } => {
                let args_file = self.find_args_file(version).await?;
                cmd.arg(format!("@{args_file}"));
            }
            Loader::Forge { version } => {
                if ProjectManager::is_modern_forge(version) {
                    let args_file = self.find_args_file(version).await?;
                    cmd.arg(format!("@{args_file}"));
                } else {
                    cmd.arg("-jar").arg("server.jar");
                }
            }
            Loader::Fabric => {
                let jar = if self.project_root.join("fabric-server-launch.jar").exists() {
                    "fabric-server-launch.jar"
                } else {
                    "server.jar"
                };
                cmd.arg("-jar").arg(jar);
            }
            _ => {
                cmd.arg("-jar").arg("server.jar");
            }
        }
        Ok(())
    }

    pub fn is_modern_forge(version: &str) -> bool {
        let parts: Vec<&str> = version.split('-').next().unwrap_or("").split('.').collect();
        let major = parts
            .first()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0);
        let minor = parts
            .get(1)
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0);

        major >= 25 || (major == 1 && minor >= 17)
    }

    async fn bridge_stdin(mut stdin: tokio::process::ChildStdin) {
        let mut reader = BufReader::new(io::stdin()).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            if stdin
                .write_all(format!("{line}\n").as_bytes())
                .await
                .is_err()
            {
                break;
            }
            let _ = stdin.flush().await;
        }
    }

    async fn find_args_file(&self, loader_version: &str) -> Result<String, StartError> {
        let libs = self.project_root.join("libraries");
        if !libs.exists() {
            return Err(StartError::NoEntryPoint);
        }

        let target_name = if cfg!(windows) {
            "win_args.txt"
        } else {
            "unix_args.txt"
        };

        let mut stack = vec![libs];
        let mut fallback = None;

        while let Some(current_dir) = stack.pop() {
            let mut entries = tokio::fs::read_dir(current_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if let Some(name) = path.file_name().and_then(|n| n.to_str())
                    && name == target_name
                {
                    let path_str = path.to_string_lossy();
                    let relative_path = path
                        .strip_prefix(&self.project_root)
                        .unwrap()
                        .to_string_lossy()
                        .to_string();

                    if path_str.contains(loader_version) {
                        return Ok(relative_path);
                    }
                    fallback = Some(relative_path);
                }
            }
        }

        fallback.ok_or(StartError::NoEntryPoint)
    }
}
