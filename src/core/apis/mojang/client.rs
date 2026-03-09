use std::sync::Arc;

pub struct MojangAPI {
    pub(super) client: Arc<reqwest::Client>,
    pub(super) base_url: String,
}

impl MojangAPI {
    pub fn new(client: Arc<reqwest::Client>) -> Self {
        Self {
            client,
            base_url: "https://piston-meta.mojang.com/mc/game".to_string(),
        }
    }
}
