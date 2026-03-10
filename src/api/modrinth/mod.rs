pub mod models;

use self::models::{ProjectResponse, VersionResponse};
use crate::api::ApiError;
use reqwest::Client;

pub struct ModrinthClient {
    client: Client,
}

impl Default for ModrinthClient {
    fn default() -> Self {
        Self {
            client: Client::builder()
                .user_agent("conduit-cli (github.com/SirCesarium/conduit-cli)")
                .build()
                .unwrap(),
        }
    }
}

impl ModrinthClient {
    pub async fn get_project(&self, id: &str) -> Result<ProjectResponse, ApiError> {
        let url = format!("https://api.modrinth.com/v2/project/{id}");
        let response = self.client.get(url).send().await?;

        if response.status() == 404 {
            return Err(ApiError::NotFound(id.to_string()));
        }

        let project = response.json::<ProjectResponse>().await?;
        Ok(project)
    }

    pub async fn get_version(&self, version_id: &str) -> Result<VersionResponse, ApiError> {
        let url = format!("https://api.modrinth.com/v2/version/{version_id}");
        let response = self.client.get(url).send().await?;

        if response.status() == 404 {
            return Err(ApiError::NotFound(version_id.to_string()));
        }

        let version = response.json::<VersionResponse>().await?;
        Ok(version)
    }

    pub async fn get_project_versions(
        &self,
        id_or_slug: &str,
        loaders: &[String],
        game_versions: &[String],
    ) -> Result<Vec<VersionResponse>, ApiError> {
        let loaders_query = serde_json::to_string(loaders).unwrap();
        let versions_query = serde_json::to_string(game_versions).unwrap();

        let url = format!(
            "https://api.modrinth.com/v2/project/{id_or_slug}/version?loaders={loaders_query}&game_versions={versions_query}"
        );

        let response = self.client.get(url).send().await?;

        if response.status() == 404 {
            return Err(ApiError::NotFound(id_or_slug.to_string()));
        }

        let versions = response.json::<Vec<VersionResponse>>().await?;
        Ok(versions)
    }
}
