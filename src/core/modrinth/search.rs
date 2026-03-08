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
        let url = Url::parse_with_params(&format!("{}/search", self.base_url), &params)
            .expect("Critical: Failed to build Modrinth search URL");

        self.client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json::<SearchResult>()
            .await
    }

    pub async fn get_suggestions(&self, input: &str) -> Vec<(String, String)> {
        let query = input.split('@').next().unwrap_or(input);

        match self.search(query, 5, 0, "relevance", None).await {
            Ok(results) => results
                .hits
                .into_iter()
                .map(|hit| (hit.title, hit.slug))
                .collect(),
            Err(_) => Vec::new(),
        }
    }
}
