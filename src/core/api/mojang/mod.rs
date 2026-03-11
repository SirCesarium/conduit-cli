pub mod models;

use self::models::{VersionDetails, VersionEntry, VersionManifest};
use crate::errors::{ConduitError, ConduitResult};
use reqwest::Client;

pub struct MojangClient {
    client: Client,
}

impl Default for MojangClient {
    fn default() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl MojangClient {
    pub async fn get_manifest(&self) -> ConduitResult<VersionManifest> {
        let url = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
        let response = self.client.get(url).send().await?;

        let manifest = response.json::<VersionManifest>().await?;
        Ok(manifest)
    }

    pub async fn get_versions_by_type(
        &self,
        version_type: &str,
    ) -> ConduitResult<Vec<VersionEntry>> {
        let manifest = self.get_manifest().await?;
        let filtered = manifest
            .versions
            .into_iter()
            .filter(|v| v.r#type == version_type)
            .collect();

        Ok(filtered)
    }

    pub async fn get_version_details(&self, version_id: &str) -> ConduitResult<VersionDetails> {
        let manifest = self.get_manifest().await?;
        let version_entry = manifest
            .versions
            .iter()
            .find(|v| v.id == version_id)
            .ok_or_else(|| ConduitError::NotFound(version_id.to_string()))?;

        let response = self.client.get(&version_entry.url).send().await?;
        let details = response.json::<VersionDetails>().await?;
        Ok(details)
    }

    pub async fn get_server_url(&self, version_id: &str) -> ConduitResult<String> {
        let details = self.get_version_details(version_id).await?;

        let server_download = details
            .downloads
            .server
            .ok_or_else(|| ConduitError::NotFound(format!("server jar for {version_id}")))?;

        Ok(server_download.url)
    }
}
