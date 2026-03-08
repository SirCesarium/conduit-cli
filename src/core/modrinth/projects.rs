use std::error::Error;

use crate::core::modrinth::models::Project;

use super::client::ModrinthAPI;

impl ModrinthAPI {
    pub async fn get_project(
        &self,
        id_or_slug: &str,
    ) -> Result<Project, Box<dyn Error>> {
        let url = format!("{}/project/{}", self.base_url, id_or_slug);

        let project: Project = self
            .client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json::<Project>()
            .await?;

        if project.project_type != "mod" {
            return Err(format!(
                "Project '{}' is a {}, but only 'mod' is supported.",
                id_or_slug, project.project_type
            )
            .into());
        }

        Ok(project)
    }
}
