use super::client::MojangAPI;
use super::models::VersionManifest;

impl MojangAPI {
    pub async fn get_manifest(&self) -> Result<VersionManifest, reqwest::Error> {
        let url = format!("{}/version_manifest_v2.json", self.base_url);

        self.client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json::<VersionManifest>()
            .await
    }
}
