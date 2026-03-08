use conduit_cli::core::io::modpack::{get_provider, PackFormat};
use conduit_cli::core::paths::CorePaths;
use console::style;
use std::path::PathBuf;

pub fn run(output: String, include_config: bool) -> Result<(), Box<dyn std::error::Error>> {
    let paths = CorePaths::from_project_dir(".")?;
    let provider = get_provider(PackFormat::Conduit);
    let output_path = PathBuf::from(&output);

    println!("{} Exporting modpack to {}...", style("📦").cyan(), style(&output).bold());

    provider.export(&paths, &output_path, include_config)?;

    let analysis = provider.analyze(&output_path)?;
    if analysis.dangerous_count > 0 || analysis.local_jars_count > 0 {
        println!("\n{} {}", style("⚠️").yellow(), style("Exported pack contains sensitive files:").bold());
        for file in analysis.suspicious_files {
            println!("  {} {}", style("-").dim(), file);
        }
        println!("{}", style("Users importing this pack will receive a security warning.\n").dim());
    }

    println!("{} Modpack exported successfully.", style("✔").green());
    Ok(())
}
