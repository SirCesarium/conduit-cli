use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ForgeMetadata {
    #[serde(rename = "groupId")]
    pub group_id: String,
    #[serde(rename = "artifactId")]
    pub artifact_id: String,
    pub versioning: ForgeVersioning,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ForgeVersioning {
    pub latest: String,
    pub release: String,
    pub versions: ForgeVersions,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ForgeVersions {
    #[serde(rename = "version")]
    pub list: Vec<String>,
}
