use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PaperBuildsResponse {
    pub project_id: String,
    pub project_name: String,
    pub version: String,
    pub builds: Vec<PaperBuild>,
}

#[derive(Debug, Deserialize)]
pub struct PaperBuild {
    pub build: u32,
    pub downloads: PaperDownloads,
}

#[derive(Debug, Deserialize)]
pub struct PaperDownloads {
    pub application: PaperApplication,
}

#[derive(Debug, Deserialize)]
pub struct PaperApplication {
    pub name: String,
    pub sha256: String,
}
