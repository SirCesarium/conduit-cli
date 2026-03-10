pub mod models;

use self::models::{PaperBuild, PaperBuildsResponse};
use crate::api::ApiError;
use reqwest::Client;

pub struct PaperClient {
    client: Client,
}

impl Default for PaperClient {
    fn default() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl PaperClient {
    pub async fn get_latest_build(&self, mc_version: &str) -> Result<PaperBuild, ApiError> {
        let url = format!("https://api.papermc.io/v2/projects/paper/versions/{mc_version}/builds");
        let response = self.client.get(url).send().await?;

        if response.status() == 404 {
            return Err(ApiError::NotFound(format!("Paper version {mc_version}")));
        }

        let data = response.json::<PaperBuildsResponse>().await?;

        data.builds
            .into_iter()
            .last()
            .ok_or_else(|| ApiError::NotFound(format!("No builds found for {mc_version}")))
    }

    pub fn build_download_url(&self, mc_version: &str, build: u32, file_name: &str) -> String {
        format!(
            "https://api.papermc.io/v2/projects/paper/versions/{0}/builds/{1}/downloads/{2}",
            mc_version, build, file_name
        )
    }
}
