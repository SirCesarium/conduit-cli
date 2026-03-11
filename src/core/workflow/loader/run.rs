use crate::core::workflow::Workflow;
use crate::domain::loader::Loader;
use crate::errors::{ConduitError, ConduitResult};
use crate::schemas::lock::Lockfile;
use std::process::Stdio;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStdin, Command};

impl Workflow {
    pub async fn run_server(&self, lock: &Lockfile) -> ConduitResult<()> {
        let mut cmd = Command::new("java");
        cmd.current_dir(&self.project_root);
        cmd.kill_on_drop(true);

        self.setup_loader_args(&mut cmd, lock).await?;
        cmd.arg("nogui");

        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::inherit());

        let mut child = cmd.spawn().map_err(ConduitError::Io)?;

        let stdin = child.stdin.take().ok_or(ConduitError::NoEntryPoint)?;
        let stdin_task = tokio::spawn(Self::bridge_stdin(stdin));

        let stdout = child.stdout.take().ok_or(ConduitError::NoEntryPoint)?;
        let mut reader = BufReader::new(stdout).lines();

        while let Ok(Some(line)) = reader.next_line().await {
            println!("{line}");
        }

        let _ = child.wait().await;
        stdin_task.abort();

        Ok(())
    }

    async fn setup_loader_args(&self, cmd: &mut Command, lock: &Lockfile) -> ConduitResult<()> {
        match &lock.instance.loader {
            Loader::Neoforge { version } => {
                let args_file = self.find_args_file(version).await?;
                cmd.arg(format!("@{args_file}"));
            }
            Loader::Forge { version } => {
                if self.is_modern_forge(version) {
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

    async fn bridge_stdin(mut child_stdin: ChildStdin) {
        let mut reader = BufReader::new(io::stdin()).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let payload = format!("{line}\n");
            if child_stdin.write_all(payload.as_bytes()).await.is_err() {
                break;
            }
            let _ = child_stdin.flush().await;
        }
    }

    async fn find_args_file(&self, loader_version: &str) -> ConduitResult<String> {
        let libs = self.project_root.join("libraries");
        if !libs.exists() {
            return Err(ConduitError::NoEntryPoint);
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
                } else if path.file_name().is_some_and(|n| n == target_name) {

                    let relative = path
                        .strip_prefix(&self.project_root)
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default();

                    if path.to_string_lossy().contains(loader_version) {
                        return Ok(relative);
                    }
                    fallback = Some(relative);
                }
            }
        }
        fallback.ok_or(ConduitError::NoEntryPoint)
    }

    fn is_modern_forge(&self, version: &str) -> bool {
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
}
