use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};

pub struct ModrinthAPI {
    pub client: reqwest::Client,
    pub base_url: String,
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
