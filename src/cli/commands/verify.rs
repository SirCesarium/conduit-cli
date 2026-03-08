use crate::cli::VerifyTarget;
use console::style;
use conduit_cli::core::paths::CorePaths;
use conduit_cli::core::project::verify::{verify_project, VerifyScope};

pub fn run(target: &VerifyTarget) -> Result<(), Box<dyn std::error::Error>> {
    let paths = CorePaths::from_project_dir(".")?;

    let scope = match target {
        VerifyTarget::Modrinth => VerifyScope::Modrinth,
        VerifyTarget::Local => VerifyScope::Local,
    };

    let report = verify_project(&paths, scope)?;

    for m in &report.missing {
        println!(
            "{} Missing file for {}: {}",
            style("✘").red(),
            style(&m.key).yellow().bold(),
            style(&m.filename).dim()
        );
    }

    for mm in &report.mismatch {
        println!(
            "{} Hash mismatch for {} ({})",
            style("✘").red(),
            style(&mm.key).yellow().bold(),
            style(&mm.filename).dim()
        );
        println!("  expected: {}", style(&mm.expected).dim());
        println!("  actual:   {}", style(&mm.actual).dim());
    }

    let label = match target {
        VerifyTarget::Modrinth => "Modrinth mods",
        VerifyTarget::Local => "local mods",
    };
    println!(
        "{} Verified {}: {} ok, {} mismatch, {} missing",
        style("✔").green(),
        label,
        report.ok,
        report.mismatch.len(),
        report.missing.len()
    );

    if !report.mismatch.is_empty() {
        return Err("Some mods failed verification".into());
    }

    Ok(())
}
