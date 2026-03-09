use super::client::ModrinthAPI;
use crate::core::apis::modrinth::models::Version;
use reqwest::Url;

impl ModrinthAPI {
    pub async fn get_versions(
        &self,
        id_or_slug: &str,
        loader: Option<&str>,
        game_version: Option<&str>,
    ) -> Result<Vec<Version>, reqwest::Error> {
        let url_str = format!("{}/project/{id_or_slug}/version", self.base_url);
        let mut url = Url::parse(&url_str).expect("Invalid base URL");

        {
            let mut pairs = url.query_pairs_mut();
            if let Some(l) = loader {
                pairs.append_pair("loaders", &format!("[\"{l}\"]"));
            }
            if let Some(gv) = game_version {
                pairs.append_pair("game_versions", &format!("[\"{gv}\"]"));
            }
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
