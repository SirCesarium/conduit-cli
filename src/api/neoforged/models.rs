use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    #[serde(rename = "groupId")]
    pub group_id: String,
    #[serde(rename = "artifactId")]
    pub artifact_id: String,
    pub versioning: Versioning,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Versioning {
    pub latest: String,
    pub release: String,
    pub versions: Versions,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Versions {
    #[serde(rename = "version")]
    pub list: Vec<String>,
}
