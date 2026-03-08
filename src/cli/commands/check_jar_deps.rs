use conduit_cli::core::mods::inspector::JarInspector;
use console::style;
use std::error::Error;

pub fn run(input: String) -> Result<(), Box<dyn Error>> {
    match JarInspector::inspect_neoforge(&input) {
        Ok(mods) => {
            println!(
                "📦 Crawling dependencies in {}:",
                style(&input).yellow()
            );

            if mods.is_empty() {
                println!("   {}", style("No external dependencies found.").dim());
            } else {
                for m in mods {
                    println!("   {} {}", style("•").magenta(), style(m).magenta().bold());
                }
            }
        }
        Err(e) => {
            return Err(format!("Failed to read JAR file: {}", e).into());
        }
    }
    Ok(())
}
