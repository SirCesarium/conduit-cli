use super::models::AddRequest;
use crate::core::error::CoreResult;
use crate::core::manager::ProjectManager;

impl ProjectManager {
    #[allow(clippy::unnecessary_wraps)]
    #[allow(clippy::unused_async)]
    pub(crate) async fn handle_add_datapack(&mut self, _request: AddRequest) -> CoreResult<()> {
        // TODO
        Ok(())
    }
}
