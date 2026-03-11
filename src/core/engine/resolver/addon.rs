use crate::core::engine::resolver::Resolver;
use crate::core::domain::addon::AddonType;
use crate::core::domain::loader::Loader;
use crate::core::domain::source::{AddonSource, Hash, SourceType};
use crate::errors::{ConduitError, ConduitResult};
use std::collections::{HashMap, HashSet};

pub struct ResolvedAddon {
    pub id: String,
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
        identifier: &str,
        mc_version: &str,
        loader: &Loader,
        expected_type: AddonType,
    ) -> ConduitResult<Vec<ResolvedAddon>> {
        let mut resolved_map: HashMap<String, ResolvedAddon> = HashMap::new();
        let mut seen_ids: HashSet<String> = HashSet::new();
        let mut to_resolve = vec![identifier.to_string()];

        while let Some(current_id) = to_resolve.pop() {
            let resolved = self
                .resolve_modrinth_addon(&current_id, mc_version, loader, expected_type.clone())
                .await?;

            if seen_ids.contains(&resolved.id) {
                continue;
            }

            let project_id = resolved.id.clone();

            for dep_id in &resolved.dependencies {
                if !seen_ids.contains(dep_id) {
                    to_resolve.push(dep_id.clone());
                }
            }

            seen_ids.insert(project_id.clone());
            resolved_map.insert(project_id, resolved);
        }

        Ok(resolved_map.into_values().collect())
    }

    pub async fn resolve_modrinth_addon(
        &self,
        id_or_slug: &str,
        mc_version: &str,
        loader: &Loader,
        expected_type: AddonType,
    ) -> ConduitResult<ResolvedAddon> {
        let project = self.api.modrinth.get_project(id_or_slug).await?;

        let mut loaders = vec![
            match loader {
                Loader::Vanilla => "minecraft",
                Loader::Fabric => "fabric",
                Loader::Forge { .. } => "forge",
                Loader::Neoforge { .. } => "neoforge",
                Loader::Paper => "paper",
                Loader::Purpur => "purpur",
            }
            .to_string(),
        ];

        if expected_type == AddonType::Datapack {
            loaders.push("datapack".to_string());
        }

        let versions = self
            .api
            .modrinth
            .get_project_versions(&project.id, &loaders, &[mc_version.to_string()])
            .await?;

        let version = versions
            .into_iter()
            .find(|v| {
                let is_datapack = v.loaders.contains(&"datapack".to_string());

                match expected_type {
                    AddonType::Datapack => is_datapack,
                    AddonType::Plugin => {
                        v.loaders.contains(&"paper".to_string())
                            || v.loaders.contains(&"spigot".to_string())
                    }
                    AddonType::Mod => !is_datapack,
                }
            })
            .ok_or_else(|| {
                ConduitError::NotFound(format!(
                    "No compatible {expected_type:?} found for {id_or_slug} on {mc_version}"
                ))
            })?;

        let file = version
            .files
            .iter()
            .find(|f| f.primary)
            .or_else(|| version.files.first())
            .ok_or_else(|| ConduitError::NotFound(format!("No files found for {id_or_slug}")))?;

        Ok(ResolvedAddon {
            id: project.id.clone(),
            slug: project.slug.clone(),
            file_name: file.filename.clone(),
            r#type: expected_type,
            loaders: vec![loader.clone()],
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
            dependencies: version
                .dependencies
                .into_iter()
                .filter(|dep| dep.dependency_type == "required")
                .filter_map(|dep| dep.project_id)
                .collect(),
        })
    }
}
