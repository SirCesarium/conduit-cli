use conduit_cli::core::events::{CoreCallbacks, CoreEvent, DownloadProgress, LogLevel};
use conduit_cli::core::installer::extra_deps::{
    ExtraDepChooser, ExtraDepDecision, ExtraDepRequest,
};
use console::{Term, style};
use indicatif::ProgressBar;
use inquire::Select;
use std::io::Write;

use crate::cli::progress::ConduitProgress;

pub struct CliUi {
    term: Term,
    download_pb: Option<ProgressBar>,
    spinner_pb: Option<ProgressBar>,
    download_filename: Option<String>,
    download_total: Option<u64>,
}

impl CliUi {
    pub fn new() -> Self {
        Self {
            term: Term::stdout(),
            download_pb: None,
            spinner_pb: None,
            download_filename: None,
            download_total: None,
        }
    }

    fn ensure_download_pb(&mut self, filename: &str, total_bytes: Option<u64>) {
        let needs_new = self
            .download_filename
            .as_ref()
            .is_none_or(|f| f != filename)
            || self.download_total != total_bytes;

        if !needs_new {
            return;
        }

        if let Some(pb) = self.download_pb.take() {
            pb.finish_and_clear();
        }

        let total = total_bytes.unwrap_or(0);
        let pb = ConduitProgress::download_style(total);
        pb.set_message(format!("{} {}", style("").cyan(), style(filename).dim()));
        self.download_pb = Some(pb);
        self.download_filename = Some(filename.to_string());
        self.download_total = total_bytes;
    }
}

impl CoreCallbacks for CliUi {
    #[allow(clippy::too_many_lines)]
    fn on_event(&mut self, event: CoreEvent) {
        match event {
            CoreEvent::WorldPreparationProgress { percentage } => {
                if let Some(pb) = &self.spinner_pb {
                    pb.set_position(u64::from(percentage));
                    pb.set_message(format!("{}", style("🌍 Preparing world...").cyan()));
                }
                return;
            }
            CoreEvent::ChatPromptRequested { sender } => {
                print!("\r{} ", style(format!("<{sender}>")).yellow());
                let _ = std::io::stdout().flush();
                return;
            }
            _ => {
                if let Some(pb) = self.download_pb.take() {
                    pb.finish_and_clear();
                    self.download_filename = None;
                }
            }
        }

        match event {
            CoreEvent::StartingServer => {
                if let Some(pb) = self.spinner_pb.take() {
                    pb.finish_and_clear();
                }
                self.spinner_pb =
                    Some(ConduitProgress::simple_spinner("Starting server...".into()));
            }
            CoreEvent::WorldPreparationStarted => {
                if let Some(pb) = self.spinner_pb.take() {
                    pb.finish_and_clear();
                }
                let pb = ConduitProgress::download_style(100);
                pb.set_message(format!(
                    "{} {}",
                    style("🌍").cyan(),
                    style("Preparing world").dim()
                ));
                self.spinner_pb = Some(pb);
            }
            CoreEvent::WorldPreparationFinished | CoreEvent::TaskFinished => {
                if let Some(pb) = self.spinner_pb.take() {
                    pb.finish_and_clear();
                }
            }
            CoreEvent::Info(msg) => {
                if let Some(title) = msg.strip_prefix("Installing ") {
                    println!(
                        "\n{} {}",
                        style("─── Installing").dim(),
                        style(title).magenta().bold()
                    );
                } else {
                    println!("{} {}", style("!").cyan(), msg);
                }
            }
            CoreEvent::SecurityWarning(message) => println!("\n{} {}\n", style(" !!! SECURITY ALERT ").on_red().white().bold(), style(message).red()),
            CoreEvent::TaskStarted(msg) => {
                if let Some(pb) = self.spinner_pb.take() {
                    pb.finish_and_clear();
                }

                let pb = ConduitProgress::simple_spinner(msg);
                self.spinner_pb = Some(pb);
            }
            CoreEvent::Warning(msg) => println!("{} {}", style("!").yellow(), msg),
            CoreEvent::Installed { slug: _, title } => {
                println!("{} Installed {}", style("✔").green(), style(title).bold());
            }
            CoreEvent::AddedAsDependency { slug } => println!(
                "{} Added {} as dependency",
                style("✔").green(),
                style(slug).bold()
            ),
            CoreEvent::AlreadyInstalled { slug } => println!(
                "{} Mod {} is already installed",
                style("!").cyan(),
                style(slug).bold()
            ),
            CoreEvent::LinkedFile { filename } => {
                println!("{} Linked {}", style("🔗").dim(), style(filename).green());
            }
            CoreEvent::Purged { slug } => {
                println!("{} Purged {}", style("🗑").dim(), style(slug).dim().italic());
            }
            CoreEvent::ChatModeStarted { sender } => {
                println!(
                    "\n{}",
                    style(format!("─── Chat Mode Enabled (Sender: {sender}) ───\n"))
                        .cyan()
                        .bold()
                );
                println!(
                    "Type {}, {} or {} to exit chat mode\n",
                    style(":exit").yellow(),
                    style(":e").yellow(),
                    style(":q").yellow()
                );
                print!("{} ", style(format!("<{sender}>")).yellow());
                let _ = std::io::stdout().flush();
            }
            CoreEvent::ChatModeStopped => {
                let _ = self.term.clear_line();
                println!("\n{}", style("─── Chat Mode Disabled ───").magenta().bold());
            }
            CoreEvent::ChatMessageSent { sender, message } => println!(
                "{} {}",
                style(format!("<{sender}>")).yellow().bold(),
                style(message).white()
            ),
            CoreEvent::ServerLogEvent {
                level,
                message,
                timestamp,
            } => {
                let time_fmt = style(timestamp).dim();
                match level {
                    LogLevel::Chat => println!(
                        "{} {} {}",
                        time_fmt,
                        style("💬").cyan(),
                        style(message).cyan().bold()
                    ),
                    LogLevel::Info => println!("{time_fmt} {message}"),
                    LogLevel::Warning => println!(
                        "{} {} {}",
                        time_fmt,
                        style("[!]").on_yellow().black(),
                        style(message).yellow()
                    ),
                    LogLevel::Error => println!(
                        "{} {} {}",
                        time_fmt,
                        style("✘").red().bold(),
                        style(message).red().bold()
                    ),
                    LogLevel::Command => println!(
                        "{} {} {}",
                        time_fmt,
                        style("⚡").magenta(),
                        style(message).dim()
                    ),
                }
            }
            CoreEvent::Error(message) => eprintln!("{} {}", style("[✘]"), message),
            CoreEvent::Success(message) => println!("{} {}", style("✔").green(), message),
            CoreEvent::ServerStopEvent(message) => println!("{}", style(message).on_red()),
            _ => {}
        }
    }

    fn on_download_progress(&mut self, progress: DownloadProgress) {
        self.ensure_download_pb(&progress.filename, progress.total_bytes);
        if let Some(pb) = &self.download_pb {
            pb.set_position(progress.bytes_downloaded);
            if Some(progress.bytes_downloaded) == progress.total_bytes {
                if let Some(pb) = self.download_pb.take() {
                    pb.finish_and_clear();
                }
                self.download_filename = None;
                self.download_total = None;
            }
        }
    }
}

impl ExtraDepChooser for CliUi {
    fn choose_extra_dep(&mut self, request: ExtraDepRequest) -> ExtraDepDecision {
        let mut options: Vec<String> = Vec::new();
        options.push(style("X Skip dependency").red().to_string());
        for c in &request.candidates {
            let entry = if c.is_exact_match {
                format!("{} {} ({})", style("!").yellow(), c.title, c.slug)
            } else {
                format!("{} ({})", c.title, c.slug)
            };
            options.push(entry);
        }
        let prompt = format!(
            "Dependency {} needed for {}:",
            style(&request.tech_id).bold().yellow(),
            style(&request.parent_filename).dim()
        );
        let selection = Select::new(&prompt, options).with_page_size(7).prompt();
        let _ = self.term.clear_last_lines(1);
        match selection {
            Ok(choice) if !choice.contains("Skip dependency") => request
                .candidates
                .iter()
                .find(|c| choice.contains(&c.slug))
                .map_or(ExtraDepDecision::Skip, |c| ExtraDepDecision::InstallSlug(c.slug.clone())),
            _ => ExtraDepDecision::Skip,
        }
    }
}
