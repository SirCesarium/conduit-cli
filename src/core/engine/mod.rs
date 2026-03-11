pub mod downloader;
pub mod io;
pub mod manager;
pub mod resolver;
pub mod store;
pub mod workflow;

use crate::api::ConduitAPI;
use crate::engine::downloader::Downloader;
use crate::engine::resolver::Resolver;
use crate::engine::store::Store;
use crate::schemas::lock::Lockfile;
use crate::schemas::manifest::Manifest;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ConduitContext {
    pub api: Arc<ConduitAPI>,
    pub store: Arc<Store>,
    pub downloader: Arc<Downloader>,
    pub resolver: Arc<Resolver>,
    pub manifest: RwLock<Manifest>,
    pub lockfile: RwLock<Lockfile>,
}

impl ConduitContext {
    pub fn new(store_root: std::path::PathBuf, manifest: Manifest, lockfile: Lockfile) -> Self {
        let api = Arc::new(ConduitAPI::new());
        let store = Arc::new(Store::new(store_root));
        let downloader = Arc::new(Downloader::new(store.clone()));
        let resolver = Arc::new(Resolver::new(api.clone()));

        Self {
            api,
            store,
            downloader,
            resolver,
            manifest: RwLock::new(manifest),
            lockfile: RwLock::new(lockfile),
        }
    }
}
