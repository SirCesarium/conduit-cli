use crate::progress::ConduitProgress;
use console::{style, Term};
use conduit_cli::core::events::{CoreCallbacks, CoreEvent, DownloadProgress};
use conduit_cli::core::installer::extra_deps::{ExtraDepChooser, ExtraDepDecision, ExtraDepRequest};
use indicatif::ProgressBar;
use inquire::Select;

pub struct CliUi {
    term: Term,
    download_pb: Option<ProgressBar>,
    download_filename: Option<String>,
    download_total: Option<u64>,
}

impl CliUi {
    pub fn new() -> Self {
        Self {
            term: Term::stdout(),
            download_pb: None,
            download_filename: None,
            download_total: None,
        }
    }

    fn ensure_download_pb(&mut self, filename: &str, total_bytes: Option<u64>) {
        let needs_new = self
            .download_filename
            .as_ref()
            .map(|f| f != filename)
            .unwrap_or(true)
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
    fn on_event(&mut self, event: CoreEvent) {
        match event {
            CoreEvent::Info(msg) => {
                if let Some(title) = msg.strip_prefix("Installing ") {
                    println!(
                        "\n{} {}",
                        style("─── Installing").dim(),
                        style(title).magenta().bold()
                    );
                } else {
                    println!("{} {}", style("ℹ").cyan(), msg);
                }
            }
            CoreEvent::Warning(msg) => println!("{} {}", style("!").yellow(), msg),
            CoreEvent::Installed { slug: _, title } => {
                println!("{} Installed {}", style("✔").green(), style(title).bold());
            }
            CoreEvent::AddedAsDependency { slug } => {
                println!(
                    "{} Added {} as dependency",
                    style("✔").green(),
                    style(slug).bold()
                );
            }
            CoreEvent::AlreadyInstalled { slug } => {
                println!(
                    "{} Mod {} is already installed",
                    style("ℹ").cyan(),
                    style(slug).bold()
                );
            }
            CoreEvent::LinkedFile { filename } => {
                println!("{} Linked {}", style("🔗").dim(), style(filename).green());
            }
            CoreEvent::Purged { slug } => {
                println!("{} Purged {}", style("🗑").dim(), style(slug).dim().italic());
            }
        }
    }

    fn on_download_progress(&mut self, progress: DownloadProgress) {
        self.ensure_download_pb(&progress.filename, progress.total_bytes);
        if let Some(pb) = &self.download_pb {
            pb.set_position(progress.bytes_downloaded);
            if let Some(total) = progress.total_bytes {
                pb.set_length(total);
            }
        }
    }
}

impl ExtraDepChooser for CliUi {
    fn choose_extra_dep(&mut self, request: ExtraDepRequest) -> ExtraDepDecision {
        let mut options: Vec<String> = Vec::new();
        options.push(style("X Skip dependency").red().to_string());

        for c in &request.candidates {
            if c.is_exact_match {
                options.push(format!(
                    "{} {} ({})",
                    style("!").yellow(),
                    c.title,
                    c.slug
                ));
            } else {
                options.push(format!("{} ({})", c.title, c.slug));
            }
        }

        let prompt = format!(
            "Dependency {} needed for {}:",
            style(&request.tech_id).bold().yellow(),
            style(&request.parent_filename).dim()
        );

        let selection = Select::new(&prompt, options)
            .with_page_size(7)
            .prompt();

        let _ = self.term.clear_last_lines(1);

        match selection {
            Ok(choice) if !choice.contains("Skip dependency") => {
                if let Some(candidate) = request
                    .candidates
                    .iter()
                    .find(|c| choice.contains(&c.slug))
                {
                    ExtraDepDecision::InstallSlug(candidate.slug.clone())
                } else {
                    ExtraDepDecision::Skip
                }
            }
            _ => ExtraDepDecision::Skip,
        }
    }
}
