use crate::core::apis::modrinth::models::Project;

use super::client::ModrinthAPI;

impl ModrinthAPI {
    pub async fn get_project(&self, id_or_slug: &str) -> Result<Project, reqwest::Error> {
        let url = format!("{}/project/{}", self.base_url, id_or_slug);

        self.client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json::<Project>()
            .await
    }
}
