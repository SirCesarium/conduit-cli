use crate::core::store::{Store, StoreError};
use futures_util::StreamExt;
use reqwest::Client;
use std::path::PathBuf;
use thiserror::Error;
use tokio::fs;
use tokio::io::AsyncWriteExt;

#[derive(Error, Debug)]
pub enum DownloadError {
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("store error: {0}")]
    Store(#[from] StoreError),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },
}

pub struct DownloadProgress {
    pub current: u64,
    pub total: u64,
}

pub struct Downloader {
    client: Client,
    store: Store,
}

impl Downloader {
    pub fn new(store: Store) -> Self {
        Self {
            client: Client::builder().user_agent("conduit-cli").build().unwrap(),
            store,
        }
    }

    pub async fn download_to_store<F>(
        &self,
        url: &str,
        expected_hash: &str,
        mut on_progress: F,
    ) -> Result<(), DownloadError>
    where
        F: FnMut(DownloadProgress) + Send,
    {
        if self.store.object_path(expected_hash).exists() {
            return Ok(());
        }

        let response = self.client.get(url).send().await?;
        let total_size = response.content_length().unwrap_or(0);

        let temp_path = std::env::temp_dir().join(format!("conduit-{}", expected_hash));
        let mut file = fs::File::create(&temp_path).await?;
        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();

        while let Some(item) = stream.next().await {
            let chunk = item?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;

            on_progress(DownloadProgress {
                current: downloaded,
                total: total_size,
            });
        }

        let actual_hash = self.store.calculate_hash(&temp_path).await?;
        if actual_hash != expected_hash {
            let _ = fs::remove_file(&temp_path).await;
            return Err(DownloadError::HashMismatch {
                expected: expected_hash.to_string(),
                actual: actual_hash,
            });
        }

        self.store.add_file(&temp_path, expected_hash).await?;
        let _ = fs::remove_file(&temp_path).await;

        Ok(())
    }
}
