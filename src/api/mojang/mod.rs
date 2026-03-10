pub mod models;

use crate::api::ApiError;
use self::models::{VersionManifest, VersionDetails, VersionEntry};
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
    pub async fn get_manifest(&self) -> Result<VersionManifest, ApiError> {
        let url = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
        let response = self.client.get(url).send().await?;
        
        let manifest = response.json::<VersionManifest>().await?;
        Ok(manifest)
    }

    pub async fn get_versions_by_type(&self, version_type: &str) -> Result<Vec<VersionEntry>, ApiError> {
        let manifest = self.get_manifest().await?;
        let filtered = manifest.versions
            .into_iter()
            .filter(|v| v.r#type == version_type)
            .collect();
            
        Ok(filtered)
    }

    pub async fn get_version_details(&self, version_id: &str) -> Result<VersionDetails, ApiError> {
        let manifest = self.get_manifest().await?;
        let version_entry = manifest.versions
            .iter()
            .find(|v| v.id == version_id)
            .ok_or_else(|| ApiError::NotFound(version_id.to_string()))?;

        let response = self.client.get(&version_entry.url).send().await?;
        let details = response.json::<VersionDetails>().await?;
        Ok(details)
    }

    pub async fn get_server_url(&self, version_id: &str) -> Result<String, ApiError> {
        let details = self.get_version_details(version_id).await?;
        
        let server_download = details.downloads.server
            .ok_or_else(|| ApiError::NotFound(format!("server jar for {version_id}")))?;
            
        Ok(server_download.url)
    }
}
