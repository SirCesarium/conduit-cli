pub mod addon;
pub mod loader;

use crate::core::ConduitContext;
use std::sync::Arc;

pub struct Resolver {
    ctx: Arc<ConduitContext>,
}

impl Resolver {
    pub fn new(ctx: Arc<ConduitContext>) -> Self {
        Self { ctx }
    }
}
