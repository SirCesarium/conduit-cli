use std::io::{Error, ErrorKind};
use std::sync::Arc;

use crate::engine::store::Store;
use crate::core::domain::source::Hash;
use crate::errors::ConduitResult;
use crate::schemas::lock::HashKind;
use crate::errors::ConduitError;
use futures_util::StreamExt;
use reqwest::Client;
use reqwest::redirect::Policy;
use tokio::fs;
use tokio::io::AsyncWriteExt;

pub struct Downloader {
    client: Client,
    store: Arc<Store>,
}

impl Downloader {
    pub fn new(store: Arc<Store>) -> Self {
        Self {
            client: Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/120.0.0.0")
                .redirect(Policy::limited(10))
                .build()
                .unwrap(),
            store,
        }
    }

    pub async fn download_to_store(
        &self,
        url: &str,
        expected_hash: Option<&Hash>,
    ) -> ConduitResult<(String, HashKind)> {
        if let Some(hash) = expected_hash {
            let (val, kind) = if let Some(h) = &hash.sha512 {
                (h, HashKind::Sha512)
            } else if let Some(h) = &hash.sha256 {
                (h, HashKind::Sha256)
            } else if let Some(h) = &hash.sha1 {
                (h, HashKind::Sha1)
            } else {
                (&String::new(), HashKind::Sha1)
            };

            if !val.is_empty() {
                let path = self.store.object_path(val, kind);
                if path.exists() {
                    return Ok((val.clone(), kind));
                }
            }
        }

        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(ConduitError::Network(
                response.error_for_status().unwrap_err(),
            ));
        }
        let temp_path = std::env::temp_dir().join(format!("conduit-{}", uuid::Uuid::new_v4()));

        {
            let mut file = fs::File::create(&temp_path).await?;
            let mut stream = response.bytes_stream();
            while let Some(item) = stream.next().await {
                let chunk = item?;
                file.write_all(&chunk).await?;
            }
            file.flush().await?;
        }

        let kind = if let Some(h) = expected_hash {
            if h.sha512.is_some() {
                HashKind::Sha512
            } else if h.sha256.is_some() {
                HashKind::Sha256
            } else {
                HashKind::Sha1
            }
        } else {
            HashKind::Sha1
        };

        let actual_hash = self.store.calculate_hash(&temp_path, kind).await?;

        if let Some(hash) = expected_hash {
            let expected_val = match kind {
                HashKind::Sha512 => hash.sha512.as_ref(),
                HashKind::Sha256 => hash.sha256.as_ref(),
                HashKind::Sha1 => hash.sha1.as_ref(),
            };

            if let Some(ev) = expected_val
                && *ev != actual_hash
            {
                let _ = fs::remove_file(&temp_path).await;
                return Err(ConduitError::HashMismatch {
                    expected: ev.clone(),
                    actual: actual_hash,
                });
            }
        }

        self.store.add_file(&temp_path, &actual_hash, kind).await?;
        let _ = fs::remove_file(&temp_path).await;

        Ok((actual_hash, kind))
    }

    pub fn download_to_store_by_hash(
        &self,
        hash: &str,
        kind: HashKind,
    ) -> ConduitResult<(String, HashKind)> {
        if hash.is_empty() {
            return Err(ConduitError::Io(Error::new(
                ErrorKind::InvalidInput,
                "empty hash",
            )));
        }

        let path = self.store.object_path(hash, kind);
        if path.exists() {
            Ok((hash.to_string(), kind))
        } else {
            Err(ConduitError::Io(Error::new(
                ErrorKind::NotFound,
                format!("hash {hash} not found"),
            )))
        }
    }
}
