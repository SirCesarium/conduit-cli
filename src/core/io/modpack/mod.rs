pub mod conduit;
pub mod metadata;
pub mod mrpack;

use crate::core::{error::CoreResult, events::CoreCallbacks, paths::CorePaths};
use std::path::Path;

pub enum PackFormat {
    Conduit,
    // MrPack,
}

pub struct PackAnalysis {
    pub files: Vec<String>,
    pub extensions: Vec<String>,
    pub dangerous_count: usize,
    pub local_jars_count: usize,
    pub suspicious_files: Vec<String>,
}

pub trait ModpackProvider {
    fn export(&self, paths: &CorePaths, output_path: &Path, include_config: bool) -> CoreResult<()>;
    fn analyze(&self, input_path: &Path) -> CoreResult<PackAnalysis>;
    
    fn import(&self, paths: &CorePaths, input_path: &Path, callbacks: &mut dyn CoreCallbacks) -> CoreResult<()>;
}

pub fn get_provider(format: &PackFormat) -> Box<dyn ModpackProvider> {
    match format {
        PackFormat::Conduit => Box::new(conduit::ConduitProvider),
        // PackFormat::MrPack => Box::new(mrpack::MrPackProvider),
    }
}
