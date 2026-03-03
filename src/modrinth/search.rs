use super::client::ModrinthAPI;
use super::models::SearchResult;
use reqwest::Url;

impl ModrinthAPI {
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
}
