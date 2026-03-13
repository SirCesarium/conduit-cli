use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModrinthIndex {
    pub game: String,
    #[serde(rename = "formatVersion")]
    pub format_version: i64,
    #[serde(rename = "versionId")]
    pub version_id: String,
    pub name: String,
    pub summary: Option<String>,
    pub files: Vec<File>,
    pub dependencies: Dependencies,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct File {
    pub path: String,
    pub hashes: Hashes,
    pub env: Env,
    pub downloads: Vec<String>,
    #[serde(rename = "fileSize")]
    pub file_size: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Hashes {
    pub sha512: String,
    pub sha1: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Env {
    pub server: String,
    pub client: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dependencies {
    #[serde(rename = "fabric-loader")]
    pub fabric_loader: Option<String>,
    pub minecraft: String,
    pub forge: Option<String>,
}
