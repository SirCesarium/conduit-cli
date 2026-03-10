#[derive(Debug, serde::Deserialize)]
pub struct PurpurVersionsResponse {
    pub versions: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct PurpurBuildsResponse {
    pub builds: PurpurBuildsList,
}

#[derive(Debug, serde::Deserialize)]
pub struct PurpurBuildsList {
    pub latest: Option<String>,
    pub all: Vec<String>,
}
