mod add;
mod install_loader;
mod start;

use crate::engine::ConduitContext;
use crate::engine::resolver::Resolver;
use crate::engine::workflow::Workflow;
use std::{path::PathBuf, sync::Arc};

pub struct ProjectManager {
    pub ctx: Arc<ConduitContext>,
    pub resolver: Resolver,
    pub project_root: PathBuf,
    pub workflow: Workflow,
}

impl ProjectManager {
    pub fn new(ctx: Arc<ConduitContext>, project_root: PathBuf) -> Self {
        let resolver = Resolver::new(ctx.api.clone());
        let workflow = Workflow::new(ctx.clone(), project_root.clone());

        Self {
            ctx,
            resolver,
            project_root,
            workflow,
        }
    }
}
