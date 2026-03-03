use crate::config::ConduitConfig;
use crate::lock::{ConduitLock, LockedMod};
use crate::modrinth::ModrinthAPI;
use crate::progress::ConduitProgress;
use async_recursion::async_recursion;
use console::style;
use futures_util::StreamExt;
use std::fs;
use std::io::Write;
use std::path::Path;

pub async fn run(api: &ModrinthAPI, input: String) -> Result<(), Box<dyn std::error::Error>> {
    let config_content = fs::read_to_string("conduit.json")
        .map_err(|_| "❌ No conduit.json found. Run 'conduit init' first.")?;
    let mut config: ConduitConfig = serde_json::from_str(&config_content)?;
    let mut lock = ConduitLock::load();

    install_recursive(api, &input, &mut config, &mut lock).await?;

    fs::write("conduit.json", serde_json::to_string_pretty(&config)?)?;
    lock.save()?;
    Ok(())
}

#[async_recursion]
async fn install_recursive(
    api: &ModrinthAPI,
    input: &str,
    config: &mut ConduitConfig,
    lock: &mut ConduitLock,
) -> Result<(), Box<dyn std::error::Error>> {
    let parts: Vec<&str> = input.split('@').collect();
    let slug_or_id = parts[0];

    let project = api.get_project(slug_or_id).await?;
    let current_slug = project.slug;

    if lock.locked_mods.contains_key(&current_slug) {
        return Ok(());
    }

    println!(
        "{} Resolving: {}",
        style("↳").dim(),
        style(&project.title).magenta().bold()
    );

    let loader_filter = config.loader.split('@').next().unwrap_or("fabric");

    let versions = api
        .get_versions(&current_slug, Some(loader_filter), Some(&config.mc_version))
        .await?;

    let selected_version = match versions.first() {
        Some(v) => v,
        None => return Err(format!("No compatible version for {}", current_slug).into()),
    };

    let file = selected_version
        .files
        .iter()
        .find(|f| f.primary)
        .unwrap_or(&selected_version.files[0]);
    let sha1 = file.hashes.get("sha1").cloned().unwrap_or_default();

    let cache_dir = dirs::data_local_dir()
        .unwrap()
        .join("conduit")
        .join("cache");
    fs::create_dir_all(&cache_dir)?;
    let cached_path = cache_dir.join(format!("{}.jar", sha1));

    let mods_dir = Path::new("mods");
    fs::create_dir_all(mods_dir)?;
    let dest_path = mods_dir.join(&file.filename);

    if !cached_path.exists() {
        let response = reqwest::get(&file.url).await?;
        let pb = ConduitProgress::download_style(response.content_length().unwrap_or(file.size));
        pb.set_message(format!("Downloading {}", style(&file.filename).yellow()));

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

    config.mods.insert(
        current_slug.clone(),
        selected_version.version_number.clone(),
    );

    let mut current_deps = Vec::new();
    for dep in &selected_version.dependencies {
        if dep.dependency_type == "required" {
            if let Some(proj_id) = &dep.project_id {
                current_deps.push(proj_id.clone());
            }
        }
    }

    lock.locked_mods.insert(
        current_slug.clone(),
        LockedMod {
            id: selected_version.project_id.clone(),
            version_id: selected_version.id.clone(),
            filename: file.filename.clone(),
            url: file.url.clone(),
            hash: sha1,
            dependencies: current_deps.clone(),
        },
    );

    for dep_id in current_deps {
        install_recursive(api, &dep_id, config, lock).await?;
    }

    println!(
        "{} Installed {}",
        style("✔").green(),
        style(&project.title).bold()
    );
    Ok(())
}
