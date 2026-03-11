pub mod fabricmc;
pub mod minecraftforge;
pub mod modrinth;
pub mod mojang;
pub mod neoforged;
pub mod papermc;
pub mod purpurmc;

use crate::core::domain::addon::Addon;
use crate::errors::ConduitResult;

pub trait AddonProvider {
    async fn get_addon(&self, id: &str) -> ConduitResult<Addon>;
    async fn get_source(&self, id: &str, version: &str) -> ConduitResult<Addon>;
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
