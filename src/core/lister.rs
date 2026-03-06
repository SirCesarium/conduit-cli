use crate::core::error::CoreResult;
use crate::core::io::project::{ConduitConfig, ConduitLock};
use crate::core::paths::CorePaths;
use std::collections::HashSet;
use std::fs;

#[derive(Debug)]
pub struct ListReport {
    pub files_on_disk: HashSet<String>,
    pub tracked_files: HashSet<String>,
    pub missing_root_slugs: Vec<String>,
    pub orphan_files: Vec<String>,
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

    for slug in config.mods.keys() {
        if let Some(locked) = lock.locked_mods.get(slug) {
            tracked_files.insert(locked.filename.clone());
            collect_tracked_files(slug, lock, &mut tracked_files);
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
    })
}

fn collect_tracked_files(slug: &str, lock: &ConduitLock, tracked: &mut HashSet<String>) {
    if let Some(locked) = lock.locked_mods.get(slug) {
        for dep_id in &locked.dependencies {
            if let Some((dep_slug, dep_info)) = lock.locked_mods.iter().find(|(_, m)| &m.id == dep_id)
            {
                tracked.insert(dep_info.filename.clone());
                collect_tracked_files(dep_slug, lock, tracked);
            }
        }
    }
}
