use crate::commands::add::install_recursive;
use crate::config::ConduitConfig;
use crate::lock::{ConduitLock, LockedMod};
use crate::modrinth::ModrinthAPI;
use crate::progress::ConduitProgress;
use console::style;
use futures_util::StreamExt;
use std::fs;
use std::io::Write;
use std::path::Path;

pub async fn run(api: &ModrinthAPI) -> Result<(), Box<dyn std::error::Error>> {
    let config_content = fs::read_to_string("conduit.json")
        .map_err(|_| "❌ No conduit.json found. Run 'conduit init' first.")?;
    let mut config: ConduitConfig = serde_json::from_str(&config_content)?;
    let mut lock = ConduitLock::load();

    println!(
        "{}",
        style("Checking for missing dependencies in lockfile...").dim()
    );

    let mods_to_check: Vec<String> = config.mods.keys().cloned().collect();
    for slug in mods_to_check {
        if !lock.locked_mods.contains_key(&slug) {
            println!(
                "{} Mod {} found in config but not in lock. Resolving...",
                style("!").yellow(),
                style(&slug).bold()
            );
            install_recursive(api, &slug, &mut config, &mut lock, true).await?;
        }
    }

    println!(
        "\n{}",
        style("─── Syncing mods from lockfile").cyan().bold()
    );

    let cache_dir = dirs::data_local_dir()
        .unwrap()
        .join("conduit")
        .join("cache");
    fs::create_dir_all(&cache_dir)?;

    let mods_dir = Path::new("mods");
    fs::create_dir_all(mods_dir)?;

    for locked_mod in lock.locked_mods.values() {
        install_from_lock(locked_mod, &cache_dir, mods_dir).await?;
    }

    fs::write("conduit.json", serde_json::to_string_pretty(&config)?)?;
    lock.save()?;

    println!("\n{} Project is up to date!", style("✔").green());
    Ok(())
}

async fn install_from_lock(
    mod_data: &LockedMod,
    cache_dir: &Path,
    mods_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let cached_path = cache_dir.join(format!("{}.jar", mod_data.hash));
    let dest_path = mods_dir.join(&mod_data.filename);

    if dest_path.exists() {
        return Ok(());
    }

    if !cached_path.exists() {
        let response = reqwest::get(&mod_data.url).await?;
        let pb = ConduitProgress::download_style(response.content_length().unwrap_or(0));
        pb.set_message(format!(
            "{} Downloading {}...",
            style("").cyan(),
            style(&mod_data.filename).dim()
        ));

        let mut cache_file = fs::File::create(&cached_path)?;
        let mut stream = response.bytes_stream();
        while let Some(item) = stream.next().await {
            let chunk = item?;
            cache_file.write_all(&chunk)?;
            pb.inc(chunk.len() as u64);
        }
        pb.finish_and_clear();
    }

    if dest_path.exists() {
        fs::remove_file(&dest_path)?;
    }
    fs::hard_link(&cached_path, &dest_path)?;

    println!(
        "{} Linked {}",
        style("🔗").dim(),
        style(&mod_data.filename).green()
    );
    Ok(())
}
