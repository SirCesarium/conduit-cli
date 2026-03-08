use super::client::ModrinthAPI;
use crate::core::modrinth::models::Version;
use std::fmt::Write;

impl ModrinthAPI {
    pub async fn get_versions(
        &self,
        id_or_slug: &str,
        loader: Option<&str>,
        game_version: Option<&str>,
    ) -> Result<Vec<Version>, reqwest::Error> {
        let mut url = format!("{}/project/{id_or_slug}/version?", self.base_url);

        if let Some(l) = loader {
            let _ = write!(url, "loaders=[\"{l}\"]&");
        }
        if let Some(gv) = game_version {
            let _ = write!(url, "game_versions=[\"{gv}\"]&");
        }

        self.client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<Version>>()
            .await
    }
}
