use std::{path::PathBuf, sync::Arc};

use crate::engine::ConduitContext;

mod loader;
mod addon;

pub struct Workflow {
    ctx: Arc<ConduitContext>,
    project_root: PathBuf,
}

impl Workflow {
    pub fn new(ctx: Arc<ConduitContext>, project_root: PathBuf) -> Self {
        Self { ctx, project_root }
    }
}
