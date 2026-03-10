pub mod models;

use self::models::ForgeMetadata;
use crate::api::ApiError;
use quick_xml::de::from_str;
use reqwest::Client;

pub struct ForgeClient {
    client: Client,
}

impl Default for ForgeClient {
    fn default() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl ForgeClient {
    pub async fn get_metadata(&self) -> Result<ForgeMetadata, ApiError> {
        let url = "https://maven.minecraftforge.net/net/minecraftforge/forge/maven-metadata.xml";
        let response = self.client.get(url).send().await?.text().await?;

        let metadata: ForgeMetadata = from_str(&response)
            .map_err(|e| ApiError::Deserialize(format!("failed to parse forge xml: {e}")))?;

        Ok(metadata)
    }

    pub async fn get_latest_version(&self, input_version: &str) -> Result<String, ApiError> {
        let meta = self.get_metadata().await?;

        if meta
            .versioning
            .versions
            .list
            .iter()
            .any(|v| v == input_version)
        {
            return Ok(input_version.to_string());
        }

        let prefix = format!("{input_version}-");

        meta.versioning
            .versions
            .list
            .into_iter()
            .rev()
            .find(|v| v.starts_with(&prefix))
            .ok_or_else(|| {
                ApiError::NotFound(format!("no forge version found for {input_version}"))
            })
    }
    pub fn build_bin_url(&self, version: &str, classifier: &str) -> String {
        let base = "https://maven.minecraftforge.net/net/minecraftforge/forge";
        format!("{base}/{version}/forge-{version}-{classifier}.jar")
    }
}
