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
        project_type: Option<&str>,
    ) -> Result<SearchResult, reqwest::Error> {
        let type_filter = project_type.map(|t| format!("\"project_type:{t}\""));

        let final_facets = match (facets, type_filter) {
            (Some(f), Some(tf)) => {
                if f.starts_with('[') && f.ends_with(']') {
                    let inner = &f[1..f.len() - 1];
                    format!("[{inner},[{tf}]]")
                } else {
                    format!("[[{tf}]]")
                }
            }
            (None, Some(tf)) => format!("[[{tf}]]"),
            (Some(f), None) => f,
            (None, None) => "[]".to_string(),
        };

        let params = vec![
            ("query", query.to_string()),
            ("limit", limit.to_string()),
            ("offset", offset.to_string()),
            ("index", index.to_string()),
            ("facets", final_facets),
        ];

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
}
