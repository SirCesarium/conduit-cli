pub mod addon;
pub mod loader;

use crate::api::ConduitAPI;
use std::sync::Arc;

pub struct Resolver {
    pub api: Arc<ConduitAPI>,
}

impl Resolver {
    pub fn new(api: Arc<ConduitAPI>) -> Self {
        Self { api }
    }
}
