use reqwest::{
    Url,
    header::{HeaderMap, HeaderValue, USER_AGENT},
};
pub mod models;

use models::*;

pub struct ModrinthAPI {
    client: reqwest::Client,
    base_url: String,
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

    pub async fn search(
        &self,
        query: &str,
        limit: i32,
        offset: i32,
        index: &str,
        facets: Option<String>,
    ) -> Result<SearchResult, reqwest::Error> {
        let mut params = vec![
            ("query", query.to_string()),
            ("limit", limit.to_string()),
            ("offset", offset.to_string()),
            ("index", index.to_string()),
        ];
        if let Some(f) = facets {
            params.push(("facets", f));
        }
        let url = Url::parse_with_params(&format!("{}/search", self.base_url), &params).unwrap();
        self.client
            .get(url)
            .send()
            .await?
            .json::<SearchResult>()
            .await
    }

    pub async fn get_versions(&self, id_or_slug: &str) -> Result<Vec<Version>, reqwest::Error> {
        let url = format!("{}/project/{}/version", self.base_url, id_or_slug);
        self.client
            .get(url)
            .send()
            .await?
            .json::<Vec<Version>>()
            .await
    }
}
