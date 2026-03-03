use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub hits: Vec<ProjectHit>,
    pub total_hits: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectHit {
    pub slug: String,
    pub title: String,
    pub project_type: String,
    pub author: String,
    pub downloads: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
    pub name: String,
    pub version_number: String,
    pub files: Vec<ModFile>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModFile {
    pub url: String,
    pub filename: String,
    pub size: u64,
    pub primary: bool,
}
