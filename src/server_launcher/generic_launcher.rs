use crate::core::events::{CoreCallbacks, CoreEvent};
use crate::server_launcher::chat::ChatManager;
use crate::server_launcher::log_processor::{handle_exit_failure, notify_startup, parse_log};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

pub struct LaunchCommand {
    pub program: String,
    pub args: Vec<String>,
    pub current_dir: PathBuf,
}

pub async fn launch_generic_server(
    cmd_config: LaunchCommand,
    callbacks: &mut dyn CoreCallbacks,
    mut show_logs: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cmd_config.program);
    cmd.args(cmd_config.args)
        .current_dir(cmd_config.current_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .stdin(Stdio::piped());

    #[cfg(unix)]
    unsafe {
        cmd.pre_exec(|| {
            libc::setsid();
            libc::setpgid(0, 0);
            Ok(())
        });
    }

    let mut child = cmd.spawn()?;

    if !show_logs {
        callbacks.on_event(CoreEvent::StartingServer);
    }

    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let mut reader = BufReader::new(stdout).lines();
    let mut stdin = child.stdin.take().ok_or("Failed to capture stdin")?;

    let mut error_buffer = Vec::new();
    let mut started = false;
    let mut preparing_world = false;
    let mut stopping = false;

    let (tx_to_server, mut rx_from_anywhere) = tokio::sync::mpsc::channel::<String>(32);
    let (tx_user_input, mut rx_user_input) = tokio::sync::mpsc::channel::<String>(32);
    let chat_manager = ChatManager::new();

    tokio::spawn(async move {
        let mut reader = BufReader::new(tokio::io::stdin()).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = tx_user_input.send(line).await;
        }
    });

    let write_handle = tokio::spawn(async move {
        while let Some(msg) = rx_from_anywhere.recv().await {
            let _ = stdin.write_all(format!("{}\n", msg).as_bytes()).await;
            let _ = stdin.flush().await;
        }
    });

    let tx_ctrlc = tx_to_server.clone();

    loop {
        tokio::select! {
            Some(line) = rx_user_input.recv() => {
                if !chat_manager.handle_input(&line, &tx_to_server, callbacks).await {
                    let _ = tx_to_server.send(line).await;
                }
            }

            line_result = reader.next_line() => {
                match line_result? {
                    Some(line) => {
                        let mut trigger_log = true;

                        if !started && line.contains("Done (") {
                            if preparing_world {
                                callbacks.on_event(CoreEvent::WorldPreparationFinished);
                                preparing_world = false;
                            }
                            notify_startup(&mut started, &mut show_logs, callbacks);
                        }

                        if !started && !preparing_world && (line.contains("Preparing level") || line.contains("Preparing start region") || line.contains("Preparing spawn area")) {
                            preparing_world = true;
                            if !show_logs {
                                callbacks.on_event(CoreEvent::WorldPreparationStarted);
                            }
                        }

                        if preparing_world && line.contains('%')
                            && let Some(pos) = line.find('%') {
                                let before = &line[..pos];
                                if let Some(num_str) = before.split_whitespace().last()
                                    && let Ok(pct) = num_str.parse::<u8>() {
                                        callbacks.on_event(CoreEvent::WorldPreparationProgress { percentage: pct });
                                        if !show_logs { trigger_log = false; }
                                    }
                            }
                        if (started || show_logs) && trigger_log {
                            let (level, message, timestamp) = parse_log(&line);
                            callbacks.on_event(CoreEvent::ServerLogEvent { level, message, timestamp });

                            if chat_manager.is_active() {
                                callbacks.on_event(CoreEvent::ChatPromptRequested { sender: chat_manager.get_name() });
                            }
                        }

                        if error_buffer.len() >= 30 { error_buffer.remove(0); }
                        error_buffer.push(line);
                    }
                    None => break,
                }
            }

            _ = tokio::signal::ctrl_c(), if !stopping => {
                if chat_manager.is_active() {
                    chat_manager.deactivate();
                    callbacks.on_event(CoreEvent::ChatModeStopped);
                } else {
                    let _ = tx_ctrlc.send("stop".into()).await;
                    callbacks.on_event(CoreEvent::ServerStopEvent("Stopping server...".into()));
                    stopping = true;
                }
            }

            status = child.wait() => {
                let exit_status = status?;
                if !exit_status.success() {
                    handle_exit_failure(callbacks, error_buffer);
                }
                break;
            }
        }
    }

    write_handle.abort();
    let _ = child.kill().await;
    Ok(())
}
