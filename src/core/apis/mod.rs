pub mod modrinth;
pub mod mojang;

use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use std::sync::Arc;

pub struct ConduitApis {
    pub modrinth: modrinth::ModrinthAPI,
    pub mojang: mojang::MojangAPI,
}

impl ConduitApis {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("Conduit-CLI (github.com/tu_usuario/conduit)"));

        let client = Arc::new(
            reqwest::Client::builder()
                .default_headers(headers)
                .build()
                .expect("Critical: Failed to create global HTTP client")
        );

        Self {
            modrinth: modrinth::ModrinthAPI::new(Arc::clone(&client)),
            mojang: mojang::MojangAPI::new(Arc::clone(&client)),
        }
    }
}

impl Default for ConduitApis {
    fn default() -> Self {
        Self::new()
    }
}