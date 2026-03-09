use crate::core::{
    context::ConduitContext,
    error::CoreResult,
    manager::add::models::{AddRequest, ResourceType},
};

pub mod add;

pub struct ProjectManager {
    ctx: ConduitContext,
}

impl ProjectManager {
    pub fn new(ctx: ConduitContext) -> Self {
        Self { ctx }
    }

    pub async fn add_resource(&mut self, request: AddRequest) -> CoreResult<()> {
        match request.r#type {
            ResourceType::Mod => self.handle_add_mod(request).await,
            ResourceType::Plugin => self.handle_add_plugin(request).await, // TODO
            ResourceType::Datapack => self.handle_add_datapack(request).await, // TODO
        }
    }
}
