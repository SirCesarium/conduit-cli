use crate::ui::CliUi;
use conduit_cli::core::events::{CoreCallbacks, CoreEvent};
use conduit_cli::core::io::modpack::{PackFormat, get_provider};
use conduit_cli::core::paths::CorePaths;
use console::style;
use inquire::Confirm;
use std::path::PathBuf;

pub fn run(input: String, yes: bool) -> Result<(), Box<dyn std::error::Error>> {
    let paths = CorePaths::from_project_dir(".")?;
    let mut ui = CliUi::new();
    let provider = get_provider(PackFormat::Conduit);
    let input_path = PathBuf::from(&input);

    if !input_path.exists() {
        return Err(format!("File not found: {}", input).into());
    }

    let analysis = provider.analyze(&input_path)?;

    if analysis.dangerous_count > 0 || analysis.local_jars_count > 0 {
        println!(
            "\n{} {}",
            style("🔍").cyan(),
            style("Security Report:").bold()
        );
        for file in &analysis.suspicious_files {
            println!("  {} {}", style("-").dim(), file);
        }
        println!();

        if analysis.dangerous_count > 0 {
            ui.on_event(CoreEvent::SecurityWarning(format!(
                "This pack contains {} executable script(s)!",
                analysis.dangerous_count
            )));
        }

        if !yes {
            let prompt: String = if analysis.dangerous_count > 0 {
                style("Are you ABSOLUTELY sure you want to proceed?").red().to_string()
            } else {
                "This pack contains local JAR files. Proceed with import?".to_string()
            };

            let confirm = Confirm::new(&prompt).with_default(false).prompt()?;

            if !confirm {
                println!("{} Import cancelled by user.", style("✘").red());
                return Ok(());
            }
        } else {
            println!(
                "{} Flag --yes detected. Skipping confirmation prompts...",
                style("!").yellow()
            );
        }
    }

    provider.import(&paths, &input_path, &mut ui)?;

    Ok(())
}
