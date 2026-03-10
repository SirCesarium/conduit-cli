pub mod models;

use self::models::{PurpurBuildsResponse, PurpurVersionsResponse};
use crate::api::ApiError;
use reqwest::Client;

pub struct PurpurClient {
    client: Client,
}

impl Default for PurpurClient {
    fn default() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl PurpurClient {
    pub async fn get_versions(&self) -> Result<Vec<String>, ApiError> {
        let url = "https://api.purpurmc.org/v2/purpur";
        let response = self.client.get(url).send().await?;
        let data = response.json::<PurpurVersionsResponse>().await?;
        Ok(data.versions)
    }

    pub async fn get_latest_build(&self, version: &str) -> Result<String, ApiError> {
        let url = format!("https://api.purpurmc.org/v2/purpur/{version}");
        let response = self.client.get(url).send().await?;

        if response.status() == 404 {
            return Err(ApiError::NotFound(format!("Purpur version {version}")));
        }

        let data = response.json::<PurpurBuildsResponse>().await?;
        data.builds
            .latest
            .ok_or_else(|| ApiError::NotFound(format!("No builds for {version}")))
    }

    pub fn build_download_url(&self, version: &str, build: &str) -> String {
        format!("https://api.purpurmc.org/v2/purpur/{version}/{build}/download")
    }
}
