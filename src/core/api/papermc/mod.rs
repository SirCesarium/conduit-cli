pub mod models;

use crate::errors::{ConduitError, ConduitResult};

use self::models::{PaperBuild, PaperBuildsResponse};
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
    pub async fn get_latest_build(&self, mc_version: &str) -> ConduitResult<PaperBuild> {
        let url = format!("https://api.papermc.io/v2/projects/paper/versions/{mc_version}/builds");
        let response = self.client.get(url).send().await?;

        if response.status() == 404 {
            return Err(ConduitError::NotFound(format!(
                "Paper version {mc_version}"
            )));
        }

        let data = response.json::<PaperBuildsResponse>().await?;

        data.builds
            .into_iter()
            .last()
            .ok_or_else(|| ConduitError::NotFound(format!("No builds found for {mc_version}")))
    }

    pub fn build_download_url(&self, mc_version: &str, build: u32, file_name: &str) -> String {
        format!(
            "https://api.papermc.io/v2/projects/paper/versions/{mc_version}/builds/{build}/downloads/{file_name}"
        )
    }
}
