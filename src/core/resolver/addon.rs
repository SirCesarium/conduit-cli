use crate::api::ApiError;
use crate::core::resolver::Resolver;
use crate::domain::addon::AddonType;
use crate::domain::loader::Loader;
use crate::domain::source::{AddonSource, Hash, SourceType};
use std::collections::HashMap;

pub struct ResolvedAddon {
    pub slug: String,
    pub file_name: String,
    pub r#type: AddonType,
    pub loaders: Vec<Loader>,
    pub download_url: String,
    pub source: AddonSource,
    pub dependencies: Vec<String>,
}

impl Resolver {
    pub async fn resolve_recursively(
        &self,
        slug: &str,
        mc_version: &str,
        loader_name: &str,
    ) -> Result<Vec<ResolvedAddon>, ApiError> {
        let mut resolved_map: HashMap<String, ResolvedAddon> = HashMap::new();
        let mut to_resolve = vec![slug.to_string()];

        while let Some(current_slug) = to_resolve.pop() {
            if resolved_map.contains_key(&current_slug) {
                continue;
            }

            let resolved = self
                .resolve_modrinth_addon(&current_slug, mc_version, loader_name)
                .await?;

            for dep_id in &resolved.dependencies {
                if !resolved_map.values().any(|r| match &r.source.r#type {
                    SourceType::Modrinth { id, .. } => id == dep_id,
                    _ => false,
                }) {
                    to_resolve.push(dep_id.clone());
                }
            }

            resolved_map.insert(current_slug, resolved);
        }

        Ok(resolved_map.into_values().collect())
    }

    pub async fn resolve_modrinth_addon(
        &self,
        slug: &str,
        mc_version: &str,
        loader_name: &str,
    ) -> Result<ResolvedAddon, ApiError> {
        let project = self.ctx.api.modrinth.get_project(slug).await?;

        if project.project_type == "resourcepack" || project.project_type == "modpack" {
            return Err(ApiError::NotFound(format!(
                "{slug} ({}) isn't supported",
                project.project_type
            )));
        }

        let versions = self
            .ctx
            .api
            .modrinth
            .get_project_versions(
                &project.id,
                &[loader_name.to_lowercase()],
                &[mc_version.to_string()],
            )
            .await?;

        let version = versions.first().ok_or_else(|| {
            ApiError::NotFound(format!(
                "No compatible version for {slug} on {mc_version}/{loader_name}"
            ))
        })?;

        let file = version
            .files
            .iter()
            .find(|f| f.primary)
            .or_else(|| version.files.first())
            .ok_or_else(|| ApiError::NotFound(format!("No files found for {slug}")))?;

        let addon_type = if version.loaders.contains(&"datapack".to_string()) {
            AddonType::Datapack
        } else if project.categories.contains(&"plugin".to_string())
            || version.loaders.contains(&"paper".to_string())
            || version.loaders.contains(&"spigot".to_string())
        {
            AddonType::Plugin
        } else {
            AddonType::Mod
        };

        let domain_loaders: Vec<Loader> = version
            .loaders
            .iter()
            .filter_map(|l| match l.as_str() {
                "neoforge" => Some(Loader::Neoforge {
                    version: mc_version.to_string(),
                }),
                "fabric" => Some(Loader::Fabric),
                "forge" => Some(Loader::Forge {
                    version: mc_version.to_string(),
                }),
                "paper" => Some(Loader::Paper),
                "purpur" => Some(Loader::Purpur),
                "minecraft" | "vanilla" => Some(Loader::Vanilla),
                _ => None,
            })
            .collect();

        let dependencies: Vec<String> = version
            .dependencies
            .iter()
            .filter(|dep| dep.dependency_type == "required")
            .filter_map(|dep| dep.project_id.clone())
            .collect();

        Ok(ResolvedAddon {
            slug: project.slug.clone(),
            file_name: file.filename.clone(),
            r#type: addon_type,
            loaders: domain_loaders,
            download_url: file.url.clone(),
            source: AddonSource {
                r#type: SourceType::Modrinth {
                    id: project.id,
                    slug: project.slug,
                },
                hash: Hash {
                    sha1: Some(file.hashes.sha1.clone()),
                    sha256: None,
                    sha512: Some(file.hashes.sha512.clone()),
                },
            },
            dependencies,
        })
    }
}
