use std::{collections::HashMap, path::PathBuf};

use uuid::Uuid;

use crate::core::{
    domain::addon::{Addon, AddonType},
    engine::{resolver::addon::ResolvedAddon, workflow::Workflow},
    schemas::lock::LockedAddon,
};
use crate::errors::{ConduitError, ConduitResult};

impl Workflow {
    pub async fn install_addon_component(
        &self,
        resolved: ResolvedAddon,
        id_map: &HashMap<String, Uuid>,
    ) -> ConduitResult<()> {
        let mut lockfile = self.ctx.lockfile.write().await;

        if lockfile
            .entries
            .values()
            .any(|e| e.metadata.slug == resolved.slug)
        {
            return Ok(());
        }

        let (hash, kind) = self
            .ctx
            .downloader
            .download_to_store(&resolved.download_url, Some(&resolved.source.hash))
            .await?;

        let rel_path = get_addon_relative_path(&resolved);

        if let Some(parent) = rel_path.parent() {
            let full_parent = self.project_root.join(parent);
            if !full_parent.exists() {
                tokio::fs::create_dir_all(&full_parent).await?;
            }
        }

        self.ctx
            .store
            .install_to_project(&hash, kind, rel_path.clone())
            .await?;

        let addon_uuid = id_map.get(&resolved.id).copied().ok_or_else(|| {
            ConduitError::Deserialize(format!(
                "Internal error: UUID not found for resolved addon '{}' ({})",
                resolved.slug, resolved.id
            ))
        })?;

        let dependency_uuids = resolved
            .dependencies
            .iter()
            .filter_map(|dep_id| id_map.get(dep_id))
            .copied()
            .collect();

        lockfile.entries.insert(
            addon_uuid,
            LockedAddon {
                metadata: Addon {
                    id: addon_uuid,
                    slug: resolved.slug,
                    file_name: resolved.file_name,
                    r#type: resolved.r#type,
                    loaders: resolved.loaders,
                    dependencies: dependency_uuids,
                },
                source: resolved.source,
            },
        );

        Ok(())
    }
}

pub fn get_addon_relative_path(resolved: &ResolvedAddon) -> PathBuf {
    match resolved.r#type {
        AddonType::Mod => PathBuf::from("mods").join(&resolved.file_name),
        AddonType::Plugin => PathBuf::from("plugins").join(&resolved.file_name),
        AddonType::Datapack => PathBuf::from("world")
            .join("datapacks")
            .join(&resolved.file_name),
    }
}
