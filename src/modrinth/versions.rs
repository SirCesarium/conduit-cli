use crate::modrinth::models::Version;

use super::client::ModrinthAPI;

impl ModrinthAPI {
    pub async fn get_versions(
        &self,
        id_or_slug: &str,
        loader: Option<&str>,
        game_version: Option<&str>,
    ) -> Result<Vec<Version>, reqwest::Error> {
        let mut url = format!("{}/project/{}/version?", self.base_url, id_or_slug);

        if let Some(l) = loader {
            url.push_str(&format!("loaders=[\"{}\"]&", l));
        }
        if let Some(gv) = game_version {
            url.push_str(&format!("game_versions=[\"{}\"]&", gv));
        }

        self.client
            .get(url)
            .send()
            .await?
            .json::<Vec<Version>>()
            .await
    }
}
