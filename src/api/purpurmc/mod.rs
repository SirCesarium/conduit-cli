pub mod models;

use crate::errors::{ConduitError, ConduitResult};

use self::models::{PurpurBuildsResponse, PurpurVersionsResponse};
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
    pub async fn get_versions(&self) -> ConduitResult<Vec<String>> {
        let url = "https://api.purpurmc.org/v2/purpur";
        let response = self.client.get(url).send().await?;
        let data = response.json::<PurpurVersionsResponse>().await?;
        Ok(data.versions)
    }

    pub async fn get_latest_build(&self, version: &str) -> ConduitResult<String> {
        let url = format!("https://api.purpurmc.org/v2/purpur/{version}");
        let response = self.client.get(url).send().await?;

        if response.status() == 404 {
            return Err(ConduitError::NotFound(format!("Purpur version {version}")));
        }

        let data = response.json::<PurpurBuildsResponse>().await?;
        data.builds
            .latest
            .ok_or_else(|| ConduitError::NotFound(format!("No builds for {version}")))
    }

    pub fn build_download_url(&self, version: &str, build: &str) -> String {
        format!("https://api.purpurmc.org/v2/purpur/{version}/{build}/download")
    }
}
