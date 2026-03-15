use std::{path::PathBuf, sync::Arc};

use conduit_cli::core::engine::{ConduitContext, manager::ProjectManager};

pub mod add;
pub mod export;
pub mod init;
pub mod install;
pub mod start;

pub struct Cmds {
    pj_manager: ProjectManager,
    ctx: Arc<ConduitContext>,
}

impl Cmds {
    pub fn new(ctx: Arc<ConduitContext>, root: PathBuf) -> Self {
        let pj_manager = ProjectManager::new(ctx.clone(), root);

        Self { pj_manager, ctx }
    }
}
