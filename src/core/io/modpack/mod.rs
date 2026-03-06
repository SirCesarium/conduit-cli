pub mod conduit;
pub mod metadata;
pub mod mrpack;

use std::path::Path;
use crate::core::error::CoreResult;

pub enum PackFormat {
    Conduit,
    // MrPack,
}

pub trait ModpackProvider {
    fn export(&self, output_path: &Path, include_config: bool) -> CoreResult<()>;
    fn import(&self, input_path: &Path, target_dir: &Path) -> CoreResult<()>;
}

pub fn get_provider(format: PackFormat) -> Box<dyn ModpackProvider> {
    match format {
        PackFormat::Conduit => Box::new(conduit::ConduitProvider),
        // PackFormat::MrPack => Box::new(mrpack::MrPackProvider),
    }
}
