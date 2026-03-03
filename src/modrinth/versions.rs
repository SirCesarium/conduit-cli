use crate::modrinth::models::Version;

use super::client::ModrinthAPI;

impl ModrinthAPI {
    pub async fn get_versions(&self, id_or_slug: &str) -> Result<Vec<Version>, reqwest::Error> {
        let url = format!("{}/project/{}/version", self.base_url, id_or_slug);
        self.client
            .get(url)
            .send()
            .await?
            .json::<Vec<Version>>()
            .await
    }
}
