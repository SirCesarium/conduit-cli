use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};

pub struct ModrinthAPI {
    pub(super) client: reqwest::Client,
    pub(super) base_url: String,
}

impl ModrinthAPI {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("Conduit-CLI"));
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();
        Self {
            client,
            base_url: "https://api.modrinth.com/v2".to_string(),
        }
    }
}
