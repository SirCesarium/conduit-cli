use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FabricInstallerEntry {
    pub version: String,
    pub stable: bool,
    pub url: String,
}
