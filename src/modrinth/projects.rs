use super::client::ModrinthAPI;
use crate::modrinth::models::Project;

impl ModrinthAPI {
    pub async fn get_project(&self, id_or_slug: &str) -> Result<Project, reqwest::Error> {
        let url = format!("{}/project/{}", self.base_url, id_or_slug);
        self.client.get(url).send().await?.json::<Project>().await
    }
}
