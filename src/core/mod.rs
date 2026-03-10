pub mod downloader;
pub mod io;
pub mod store;
pub mod resolver;

use crate::api::ConduitAPI;
use crate::core::downloader::Downloader;
use crate::core::store::Store;
use std::sync::Arc;

pub struct ConduitContext {
    pub api: Arc<ConduitAPI>,
    pub store: Arc<Store>,
    pub downloader: Arc<Downloader>,
}

impl ConduitContext {
    pub fn new(store_root: std::path::PathBuf) -> Self {
        let api = Arc::new(ConduitAPI::new());
        let store = Arc::new(Store::new(store_root));
        let downloader = Arc::new(Downloader::new((*store).clone()));

        Self {
            api,
            store,
            downloader,
        }
    }
}
