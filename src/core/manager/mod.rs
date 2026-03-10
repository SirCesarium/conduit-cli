mod install_loader;
mod start;

use std::{path::PathBuf, sync::Arc};

use crate::core::ConduitContext;
use crate::core::resolver::Resolver;

pub struct ProjectManager {
    ctx: Arc<ConduitContext>,
    resolver: Resolver,
    project_root: PathBuf,
}

impl ProjectManager {
    pub fn new(ctx: Arc<ConduitContext>, project_root: PathBuf) -> Self {
        let resolver = Resolver::new(ctx.clone());
        Self {
            ctx,
            resolver,
            project_root,
        }
    }
}
