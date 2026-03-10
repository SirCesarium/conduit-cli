pub mod models;

use crate::api::ApiError;
use self::models::FabricInstallerEntry;
use reqwest::Client;

pub struct FabricClient {
    client: Client,
}

impl Default for FabricClient {
    fn default() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl FabricClient {
    pub async fn get_latest_installer(&self) -> Result<String, ApiError> {
        let url = "https://meta.fabricmc.net/v2/versions/installer";
        let entries: Vec<FabricInstallerEntry> = self.client.get(url).send().await?.json().await?;

        entries
            .into_iter()
            .find(|e| e.stable)
            .map(|e| e.version)
            .ok_or_else(|| ApiError::NotFound("stable fabric installer".to_string()))
    }

    pub fn build_installer_url(&self, version: &str) -> String {
        format!(
            "https://maven.fabricmc.net/net/fabricmc/fabric-installer/{0}/fabric-installer-{0}.jar",
            version
        )
    }
}
