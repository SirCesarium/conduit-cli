use serde::{Deserialize, Serialize};
use strum::Display;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(tag = "type", rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum SourceType {
    Modrinth { id: String, slug: String },
    Local { path: PathBuf },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hash {
    pub sha1: Option<String>,
    pub sha256: Option<String>,
    pub sha512: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddonSource {
    pub r#type: SourceType,
    pub hash: Hash,
}
