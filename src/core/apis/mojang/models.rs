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
pub struct VersionEntry {
    pub id: String,
    pub r#type: String, // release o snapshot
    pub url: String,    // URL al JSON detallado de la versión
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionDetails {
    pub id: String,
    pub downloads: Downloads,
    #[serde(rename = "javaVersion")]
    pub java_version: JavaVersion,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Downloads {
    pub server: Option<FileInfo>,
    pub client: Option<FileInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JavaVersion {
    #[serde(rename = "majorVersion")]
    pub major: u32,
}
