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
    fn get_addon(&self, id: &str)
    -> impl std::future::Future<Output = ConduitResult<Addon>> + Send;
    fn get_source(
        &self,
        id: &str,
        version: &str,
    ) -> impl std::future::Future<Output = ConduitResult<Addon>> + Send;
}

#[derive(Default)]
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
        Self::default()
    }
}
