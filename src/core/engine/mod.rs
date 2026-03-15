pub mod archive;
pub mod downloader;
pub mod io;
pub mod manager;
pub mod resolver;
pub mod store;
pub mod workflow;

use crate::core::api::ConduitAPI;
use crate::core::engine::downloader::Downloader;
use crate::core::engine::resolver::Resolver;
use crate::core::engine::store::Store;
use crate::core::schemas::include::ConduitInclude;
use crate::core::schemas::lock::Lockfile;
use crate::core::schemas::manifest::Manifest;
use crate::paths::ConduitPaths;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ConduitContext {
    pub api: Arc<ConduitAPI>,
    pub store: Arc<Store>,
    pub downloader: Arc<Downloader>,
    pub resolver: Arc<Resolver>,
    pub manifest: RwLock<Manifest>,
    pub lockfile: RwLock<Lockfile>,
    pub includefile: RwLock<ConduitInclude>,
    pub paths: ConduitPaths,
}

impl ConduitContext {
    pub fn new(
        paths: ConduitPaths,
        manifest: Manifest,
        lockfile: Lockfile,
        includefile: ConduitInclude,
    ) -> Self {
        let api = Arc::new(ConduitAPI::new());
        let store = Arc::new(Store::new(paths.clone().store));
        let downloader = Arc::new(Downloader::new(store.clone()));
        let resolver = Arc::new(Resolver::new(api.clone()));

        Self {
            api,
            store,
            downloader,
            resolver,
            manifest: RwLock::new(manifest),
            lockfile: RwLock::new(lockfile),
            includefile: RwLock::new(includefile),
            paths,
        }
    }
}
