use crate::modrinth::ModrinthAPI;
use crate::progress::ConduitProgress;
use console::style;
use futures_util::StreamExt;
use std::fs;
use std::io::Write;

pub async fn run(api: &ModrinthAPI, input: String) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{} {} {}",
        style("⚡").yellow(),
        style("Channeling").cyan(),
        style(&input).bold()
    );

    let versions = api.get_versions(&input).await?;

    let latest_version = versions.first().ok_or("❌ No versions available")?;

    let file = latest_version
        .files
        .iter()
        .find(|f| f.primary)
        .unwrap_or(&latest_version.files[0]);

    fs::create_dir_all("mods")?;
    let dest_path = format!("mods/{}", file.filename);

    let response = reqwest::get(&file.url).await?;
    let total_size = response.content_length().unwrap_or(file.size);

    let pb = ConduitProgress::download_style(total_size);
    pb.set_message(format!("Downloading {}", style(&file.filename).yellow()));

    let mut file_on_disk = fs::File::create(&dest_path)?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item?;
        file_on_disk.write_all(&chunk)?;
        downloaded += chunk.len() as u64;
        pb.set_position(downloaded);
    }

    pb.finish_with_message(format!(
        "{} Installed {} in /mods",
        style("⚡").yellow(),
        style(&input).green().bold()
    ));

    Ok(())
}
