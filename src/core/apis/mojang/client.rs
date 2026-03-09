use super::models::{VersionDetails, VersionManifest};
use reqwest::Client;

pub struct MojangAPI {
    client: Client,
    manifest_url: String,
}

impl Default for MojangAPI {
    fn default() -> Self {
        Self {
            client: Client::new(),
            manifest_url: "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json"
                .to_string(),
        }
    }
}
impl MojangAPI {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            manifest_url: "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json"
                .to_string(),
        }
    }

    pub async fn get_manifest(&self) -> Result<VersionManifest, reqwest::Error> {
        self.client
            .get(&self.manifest_url)
            .send()
            .await?
            .json::<VersionManifest>()
            .await
    }

    pub async fn get_version_details(
        &self,
        version_id: &str,
    ) -> Result<VersionDetails, Box<dyn std::error::Error>> {
        let manifest = self.get_manifest().await?;

        let entry = manifest
            .versions
            .iter()
            .find(|v| v.id == version_id)
            .ok_or(format!("Version {version_id} not found in Mojang manifest"))?;

        let details = self
            .client
            .get(&entry.url)
            .send()
            .await?
            .json::<VersionDetails>()
            .await?;

        Ok(details)
    }

    pub async fn get_latest_server_url(&self) -> Result<String, Box<dyn std::error::Error>> {
        let manifest = self.get_manifest().await?;
        let latest_release = manifest.latest.release;

        let details = self.get_version_details(&latest_release).await?;

        details
            .downloads
            .server
            .map(|s| s.url)
            .ok_or("Latest release does not have a server download".into())
    }
}
