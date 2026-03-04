use crate::core::error::CoreResult;
use crate::core::events::{CoreCallbacks, DownloadProgress};
use futures_util::StreamExt;
use std::fs;
use std::io::Write;
use std::path::Path;

pub async fn download_to_path(
    url: &str,
    dest_path: &Path,
    filename: &str,
    callbacks: &mut dyn CoreCallbacks,
) -> CoreResult<()> {
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let response = reqwest::get(url).await?;
    let total = response.content_length();

    let mut file = fs::File::create(dest_path)?;
    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;

    while let Some(item) = stream.next().await {
        let chunk = item?;
        file.write_all(&chunk)?;
        downloaded += chunk.len() as u64;
        callbacks.on_download_progress(DownloadProgress {
            bytes_downloaded: downloaded,
            total_bytes: total,
            filename: filename.to_string(),
        });
    }

    if let Some(t) = total {
        callbacks.on_download_progress(DownloadProgress {
            bytes_downloaded: t,
            total_bytes: Some(t),
            filename: filename.to_string(),
        });
    }

    Ok(())
}
