pub mod models;

use self::models::Metadata;
use crate::api::ApiError;
use quick_xml::de::from_str;
use reqwest::Client;

pub struct NeoForgeClient {
    client: Client,
}

impl Default for NeoForgeClient {
    fn default() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl NeoForgeClient {
    pub async fn get_metadata(&self) -> Result<Metadata, ApiError> {
        let url = "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml";
        let response = self.client.get(url).send().await?.text().await?;

        let metadata: Metadata = from_str(&response)
            .map_err(|e| ApiError::Deserialize(format!("failed to parse neoforge xml: {e}")))?;

        Ok(metadata)
    }

    pub async fn get_latest_version(&self) -> Result<String, ApiError> {
        let meta = self.get_metadata().await?;
        Ok(meta.versioning.release)
    }

    pub fn build_bin_url(&self, version: &str, classifier: &str) -> String {
        let base = "https://maven.neoforged.net/releases";
        let group = "net/neoforged/neoforge";
        let artifact = "neoforge";

        format!("{base}/{group}/{version}/{artifact}-{version}-{classifier}.jar")
    }
}
