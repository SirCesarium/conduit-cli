use crate::config::ConduitConfig;
use crate::core::error::CoreResult;
use crate::core::events::{CoreCallbacks, CoreEvent};
use crate::core::io::ConduitLock;
use crate::core::paths::CorePaths;
use std::collections::HashSet;
use std::fs;

pub struct RemoveReport {
    pub removed_slug: String,
    pub purged_slugs: Vec<String>,
}

pub fn remove_mod(
    paths: &CorePaths,
    slug: &str,
    config: &mut ConduitConfig,
    lock: &mut ConduitLock,
    callbacks: &mut dyn CoreCallbacks,
) -> CoreResult<Option<RemoveReport>> {
    if !config.mods.contains_key(slug) {
        return Ok(None);
    }

    config.mods.remove(slug);

    let mut mods_to_keep = HashSet::new();
    for root_slug in config.mods.keys() {
        collect_dependencies(root_slug, lock, &mut mods_to_keep);
    }

    let mut purged = Vec::new();
    let all_locked_slugs: Vec<String> = lock.locked_mods.keys().cloned().collect();
    for locked_slug in all_locked_slugs {
        if !mods_to_keep.contains(&locked_slug)
            && let Some(mod_data) = lock.locked_mods.remove(&locked_slug) {
                let dest_path = paths.mods_dir().join(&mod_data.filename);
                if dest_path.exists() {
                    fs::remove_file(dest_path)?;
                }
                purged.push(locked_slug.clone());
                callbacks.on_event(CoreEvent::Purged { slug: locked_slug });
            }
    }

    Ok(Some(RemoveReport {
        removed_slug: slug.to_string(),
        purged_slugs: purged,
    }))
}

fn collect_dependencies(slug: &str, lock: &ConduitLock, kept: &mut HashSet<String>) {
    if kept.contains(slug) {
        return;
    }
    if let Some(mod_data) = lock.locked_mods.get(slug) {
        kept.insert(slug.to_string());
        for dep_id in &mod_data.dependencies {
            if let Some((dep_slug, _)) = lock.locked_mods.iter().find(|(_, m)| &m.id == dep_id) {
                collect_dependencies(dep_slug, lock, kept);
            }
        }
    }
}
