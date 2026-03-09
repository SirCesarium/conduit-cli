use crate::core::{
    context::ConduitContext,
    error::{CoreError, CoreResult},
    io::project::manifest::LoaderInfo,
    manager::add::models::RemoteSource,
};

pub struct ModResolution {
    pub slug: String,
    pub id: String,
    pub version_id: String,
    pub filename: String,
    pub download_url: String,
    pub hash: String,
    pub deps: Vec<RemoteSource>,
}

pub async fn resolve_remote_mod(
    ctx: &ConduitContext,
    source: &RemoteSource,
    minecraft_version: &str,
    loader_info: &LoaderInfo,
) -> CoreResult<ModResolution> {
    match source {
        RemoteSource::Modrinth {
            slug,
            version: version_req,
        } => {
            let project = ctx
                .api
                .modrinth
                .get_project(slug)
                .await
                .map_err(|_| CoreError::ProjectNotFound { slug: slug.clone() })?;

            let versions = ctx
                .api
                .modrinth
                .get_versions(
                    &project.id,
                    Some(&loader_info.loader),
                    Some(minecraft_version),
                )
                .await?;

            let selected = if let Some(req) = version_req {
                versions
                    .into_iter()
                    .find(|v| &v.version_number == req)
                    .ok_or_else(|| {
                        CoreError::RuntimeError(format!("Versión {req} no encontrada para {slug}"))
                    })?
            } else {
                versions.into_iter().next().ok_or_else(|| {
                    CoreError::RuntimeError(format!(
                        "No hay versiones compatibles para {slug} en MC {minecraft_version}"
                    ))
                })?
            };

            let file = selected
                .files
                .iter()
                .find(|f| f.primary)
                .or_else(|| selected.files.first())
                .ok_or_else(|| {
                    CoreError::RuntimeError(format!(
                        "La versión {} de {} no tiene archivos",
                        selected.version_number, slug
                    ))
                })?;

            let deps = selected
                .dependencies
                .into_iter()
                .filter(|d| d.dependency_type == "required")
                .filter_map(|d| {
                    d.project_id.map(|id| RemoteSource::Modrinth {
                        slug: id,
                        version: None,
                    })
                })
                .collect();

            let hash = file
                .hashes
                .get("sha512")
                .or_else(|| file.hashes.get("sha1"))
                .cloned()
                .unwrap_or_default();

            Ok(ModResolution {
                slug: project.slug,
                id: project.id,
                version_id: selected.id,
                filename: file.filename.clone(),
                download_url: file.url.clone(),
                hash,
                deps,
            })
        }
    }
}
