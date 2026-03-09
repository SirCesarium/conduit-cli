use crate::core::apis::modrinth::ModrinthAPI;

impl ModrinthAPI {
    pub async fn get_suggestions(
        &self,
        input: &str,
        project_type: Option<&str>,
    ) -> Vec<(String, String)> {
        let query = input.split('@').next().unwrap_or(input);

        match self
            .search(query, 5, 0, "relevance", None, project_type)
            .await
        {
            Ok(results) => results
                .hits
                .into_iter()
                .map(|hit| (hit.title, hit.slug))
                .collect(),
            Err(_) => Vec::new(),
        }
    }
}
