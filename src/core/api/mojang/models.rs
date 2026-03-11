use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionManifest {
    pub latest: LatestVersions,
    pub versions: Vec<VersionEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LatestVersions {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionEntry {
    pub id: String,
    pub r#type: String,
    pub url: String,
    pub time: String,
    pub release_time: String,
    pub sha1: String,
    pub compliance_level: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionDetails {
    pub id: String,
    pub r#type: String,
    pub downloads: Downloads,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Downloads {
    pub client: Option<DownloadArtifact>,
    pub server: Option<DownloadArtifact>,
    pub server_mappings: Option<DownloadArtifact>,
    pub client_mappings: Option<DownloadArtifact>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadArtifact {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}
