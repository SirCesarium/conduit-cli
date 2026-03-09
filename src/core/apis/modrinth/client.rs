use std::sync::Arc;

pub struct ModrinthAPI {
    pub(super) client: Arc<reqwest::Client>,
    pub(super) base_url: String,
}

impl ModrinthAPI {
    pub fn new(client: Arc<reqwest::Client>) -> Self {
        Self {
            client,
            base_url: "https://api.modrinth.com/v2".to_string(),
        }
    }
}
