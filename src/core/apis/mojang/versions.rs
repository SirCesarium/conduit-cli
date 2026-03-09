use super::client::MojangAPI;
use super::models::VersionDetails;
use std::error::Error;

impl MojangAPI {
    pub async fn get_version_details(
        &self,
        version_id: &str,
    ) -> Result<VersionDetails, Box<dyn Error>> {
        let manifest = self.get_manifest().await?;

        let entry = manifest
            .versions
            .iter()
            .find(|v| v.id == version_id)
            .ok_or_else(|| format!("Version {version_id} not found in Mojang manifest"))?;

        let details = self
            .client
            .get(&entry.url)
            .send()
            .await?
            .error_for_status()?
            .json::<VersionDetails>()
            .await?;

        Ok(details)
    }

    pub async fn get_latest_server_url(&self) -> Result<String, Box<dyn Error>> {
        let manifest = self.get_manifest().await?;
        let latest_release = manifest.latest.release;

        let details = self.get_version_details(&latest_release).await?;

        details
            .downloads
            .server
            .map(|s| s.url)
            .ok_or_else(|| "Latest release does not have a server download".into())
    }
}
