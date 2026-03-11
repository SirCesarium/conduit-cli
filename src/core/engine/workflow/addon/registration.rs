use std::collections::HashMap;

use uuid::Uuid;

use crate::{
    core::{
        domain::{addon::AddonType, source::SourceType},
        engine::{io::TomlFile, resolver::addon::ResolvedAddon, workflow::Workflow},
    },
    errors::ConduitResult,
};

impl Workflow {
    pub async fn prepare_addon_uuids(
        &self,
        resolved_list: &[ResolvedAddon],
    ) -> ConduitResult<HashMap<String, Uuid>> {
        let mut id_map = HashMap::new();
        let lockfile = self.ctx.lockfile.read().await;

        for (uuid, entry) in &lockfile.entries {
            if let SourceType::Modrinth { id, .. } = &entry.source.r#type {
                id_map.insert(id.clone(), *uuid);
            }
        }

        for resolved in resolved_list {
            id_map
                .entry(resolved.id.clone())
                .or_insert_with(Uuid::new_v4);
        }
        Ok(id_map)
    }

    pub async fn update_manifest_addons(&self, primary: &ResolvedAddon) -> ConduitResult<()> {
        let mut manifest = self.ctx.manifest.write().await;
        let slug = primary.slug.clone();
        let version = "*".to_string();

        match primary.r#type {
            AddonType::Mod => manifest.mods.insert(slug, version),
            AddonType::Plugin => manifest.plugins.insert(slug, version),
            AddonType::Datapack => manifest.datapacks.insert(slug, version),
        };

        manifest.save(self.ctx.paths.manifest()).await
    }
}
