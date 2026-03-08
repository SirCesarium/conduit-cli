use crate::core::error::CoreResult;
use crate::core::io::project::lock::LockedMod;
use crate::core::io::project::{ConduitConfig, ConduitLock};
use crate::core::paths::CorePaths;
use std::collections::{HashMap, HashSet};
use std::fs;

#[derive(Debug)]
pub struct ListReport {
    pub files_on_disk: HashSet<String>,
    pub tracked_files: HashSet<String>,
    pub missing_root_slugs: Vec<String>,
    pub orphan_files: Vec<String>,
    pub dependency_map: HashMap<String, Vec<(String, LockedMod)>>,
}

pub fn build_list_report(
    paths: &CorePaths,
    config: &ConduitConfig,
    lock: &ConduitLock,
) -> CoreResult<ListReport> {
    let mut files_on_disk = HashSet::new();

    if paths.mods_dir().exists() {
        for entry in fs::read_dir(paths.mods_dir())? {
            let entry = entry?;
            if let Some(name) = entry.file_name().to_str() {
                files_on_disk.insert(name.to_string());
            }
        }
    }

    let mut missing_root_slugs = Vec::new();
    let mut tracked_files = HashSet::new();
    let mut dependency_map = HashMap::new();

    for slug in config.mods.keys() {
        if let Some(locked) = lock.locked_mods.get(slug) {
            tracked_files.insert(locked.filename.clone());
            collect_tracked_data(slug, lock, &mut tracked_files, &mut dependency_map);
        } else {
            missing_root_slugs.push(slug.clone());
        }
    }

    let orphan_files: Vec<String> = files_on_disk
        .iter()
        .filter(|f| !tracked_files.contains(*f))
        .cloned()
        .collect();

    Ok(ListReport {
        files_on_disk,
        tracked_files,
        missing_root_slugs,
        orphan_files,
        dependency_map,
    })
}

fn collect_tracked_data(
    slug: &str,
    lock: &ConduitLock,
    tracked: &mut HashSet<String>,
    map: &mut HashMap<String, Vec<(String, LockedMod)>>,
) {
    if map.contains_key(slug) {
        return;
    }

    if let Some(locked) = lock.locked_mods.get(slug) {
        let mut deps_found = Vec::new();
        for dep_id in &locked.dependencies {
            if let Some((dep_slug, dep_info)) =
                lock.locked_mods.iter().find(|(_, m)| &m.id == dep_id)
            {
                tracked.insert(dep_info.filename.clone());
                deps_found.push((dep_slug.clone(), dep_info.clone()));
            }
        }

        if !deps_found.is_empty() {
            map.insert(slug.to_string(), deps_found.clone());
            for (dep_slug, _) in deps_found {
                collect_tracked_data(&dep_slug, lock, tracked, map);
            }
        }
    }
}
