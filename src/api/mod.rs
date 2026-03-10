pub mod fabricmc;
pub mod minecraftforge;
pub mod modrinth;
pub mod mojang;
pub mod neoforged;
pub mod papermc;
pub mod purpurmc;

use crate::domain::addon::Addon;
use crate::domain::source::AddonSource;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("network request failed: {0}")]
    Network(#[from] reqwest::Error),

    #[error("resource not found: {0}")]
    NotFound(String),

    #[error("failed to deserialize response: {0}")]
    Deserialize(String),
}

pub trait AddonProvider {
    async fn get_addon(&self, id: &str) -> Result<Addon, ApiError>;
    async fn get_source(&self, id: &str, version: &str) -> Result<AddonSource, ApiError>;
}

pub struct ConduitAPI {
    pub modrinth: modrinth::ModrinthClient,
    pub mojang: mojang::MojangClient,
    pub neoforged: neoforged::NeoForgeClient,
    pub fabricmc: fabricmc::FabricClient,
    pub papermc: papermc::PaperClient,
    pub minecraftforge: minecraftforge::ForgeClient,
    pub purpurmc: purpurmc::PurpurClient,
}

impl ConduitAPI {
    pub fn new() -> Self {
        Self {
            modrinth: modrinth::ModrinthClient::default(),
            mojang: mojang::MojangClient::default(),
            neoforged: neoforged::NeoForgeClient::default(),
            fabricmc: fabricmc::FabricClient::default(),
            papermc: papermc::PaperClient::default(),
            minecraftforge: minecraftforge::ForgeClient::default(),
            purpurmc: purpurmc::PurpurClient::default(),
        }
    }
}
