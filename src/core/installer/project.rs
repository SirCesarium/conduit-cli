use crate::core::error::{CoreError, CoreResult};
use crate::core::installer::extra_deps::{ExtraDepsPolicy, InstallerUi};
use crate::core::installer::resolve::{InstallOptions, install_mod};
use crate::core::installer::sync::sync_from_lock;
use crate::core::io::project::lock::{LockedMod, ModSide};
use crate::core::io::project::{ConduitConfig, ConduitLock, ProjectFiles};
use crate::core::mods::local::add_local_mods_to_project;
use crate::core::paths::CorePaths;
use crate::core::modrinth::ModrinthAPI;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

pub struct InstallProjectOptions {
    pub extra_deps_policy: ExtraDepsPolicy,
    pub strict: bool,
    pub force: bool,
}

#[derive(Debug, Default, Clone)]
pub struct SyncProjectReport {
    pub pruned_files: Vec<String>,
}

impl Default for InstallProjectOptions {
    fn default() -> Self {
        Self {
            extra_deps_policy: ExtraDepsPolicy::Skip,
            strict: false,
            force: false,
        }
    }
}

pub async fn add_mods_to_project(
    api: &ModrinthAPI,
    paths: &CorePaths,
    inputs: Vec<String>,
    explicit_deps: Vec<String>,
    ui: &mut dyn InstallerUi,
    options: InstallProjectOptions,
) -> CoreResult<()> {
    let mut local_paths = Vec::new();
    let mut root_modrinth = Vec::new();
    let mut dep_modrinth = Vec::new();

    for input in inputs {
        if let Some(path_str) = input.strip_prefix("f:").or_else(|| input.strip_prefix("file:")) {
            local_paths.push(PathBuf::from(path_str));
        } else {
            root_modrinth.push(input);
        }
    }

    for dep in explicit_deps {
        if let Some(path_str) = dep.strip_prefix("f:").or_else(|| dep.strip_prefix("file:")) {
            local_paths.push(PathBuf::from(path_str));
        } else {
            dep_modrinth.push(dep);
        }
    }

    if !local_paths.is_empty() {
        add_local_mods_to_project(paths, local_paths)?;
    }

    if !root_modrinth.is_empty() || !dep_modrinth.is_empty() {
        let mut config = ProjectFiles::load_manifest(paths)?;
        let mut lock = ProjectFiles::load_lock(paths)?;

        for slug in root_modrinth.iter().chain(dep_modrinth.iter()) {
            if let Err(e) = api.get_project(slug).await {
                if e.status() == Some(reqwest::StatusCode::NOT_FOUND) {
                    return Err(CoreError::ProjectNotFound { slug: slug.clone() });
                }
                return Err(e.into());
            }
        }

        for slug in root_modrinth {
            install_mod(api, paths, &slug, &mut config, &mut lock, ui, 
                InstallOptions { is_root: true, extra_deps_policy: options.extra_deps_policy.clone() }
            ).await?;
        }

        for slug in dep_modrinth {
            install_mod(api, paths, &slug, &mut config, &mut lock, ui, 
                InstallOptions { is_root: false, extra_deps_policy: options.extra_deps_policy.clone() }
            ).await?;
        }

        ProjectFiles::save_manifest(paths, &config)?;
        ProjectFiles::save_lock(paths, &lock)?;
    }

    Ok(())
}

pub async fn sync_project(
    api: &ModrinthAPI,
    paths: &CorePaths,
    ui: &mut dyn InstallerUi,
    options: InstallProjectOptions,
) -> CoreResult<SyncProjectReport> {
    let mut config = ProjectFiles::load_manifest(paths)?;
    let mut lock = ProjectFiles::load_lock(paths)?;

    if options.force {
        lock = rebuild_lock_from_config(api, paths, ui, &config, &lock, &options).await?;
    }

    let mods_to_check: Vec<String> = config
        .mods
        .iter()
        .filter(|(_k, v)| v != &"local")
        .map(|(k, _v)| k.clone())
        .collect();
    for slug in mods_to_check {
        if !lock.locked_mods.contains_key(&slug) {
            let input = if let Some(version) = config.mods.get(&slug) {
                if version == "latest" {
                    slug.clone()
                } else {
                    format!("{slug}@{version}")
                }
            } else {
                slug.clone()
            };

            install_mod(
                api,
                paths,
                &input,
                &mut config,
                &mut lock,
                ui,
                InstallOptions {
                    is_root: true,
                    extra_deps_policy: options.extra_deps_policy.clone(),
                },
            )
            .await?;
        }
    }

    sync_from_lock(
        paths,
        lock.locked_mods.values().filter(|m| m.url != "local"),
        ui,
    )
    .await?;

    let mut report = SyncProjectReport::default();
    if options.strict {
        report.pruned_files = prune_unmanaged_mods(paths, &config, &lock)?;
    }

    ProjectFiles::save_manifest(paths, &config)?;
    ProjectFiles::save_lock(paths, &lock)?;

    Ok(report)
}

async fn rebuild_lock_from_config(
    api: &ModrinthAPI,
    paths: &CorePaths,
    ui: &mut dyn InstallerUi,
    config: &ConduitConfig,
    existing_lock: &ConduitLock,
    options: &InstallProjectOptions,
) -> CoreResult<ConduitLock> {
    let mut new_lock = ConduitLock {
        conduit_version: env!("CARGO_PKG_VERSION").to_string(),
        version: existing_lock.version,
        locked_mods: existing_lock
            .locked_mods
            .iter()
            .filter(|(_k, v)| v.url == "local")
            .map(|(k, v)| {
                (
                    k.clone(),
                    LockedMod {
                        id: v.id.clone(),
                        version_id: v.version_id.clone(),
                        filename: v.filename.clone(),
                        url: v.url.clone(),
                        hash: v.hash.clone(),
                        dependencies: v.dependencies.clone(),
                        side: ModSide::Both // TODO: update crawler to use real mod side here
                    },
                )
            })
            .collect(),
        loader_version: existing_lock.loader_version.clone(),
    };

    let mut dummy_config = config.clone();

    for (slug, version) in &config.mods {
        if version == "local" {
            continue;
        }

        let input = if version == "latest" {
            slug.clone()
        } else {
            format!("{slug}@{version}")
        };

        install_mod(
            api,
            paths,
            &input,
            &mut dummy_config,
            &mut new_lock,
            ui,
            InstallOptions {
                is_root: false,
                extra_deps_policy: options.extra_deps_policy.clone(),
            },
        )
        .await?;
    }

    Ok(new_lock)
}

fn prune_unmanaged_mods(
    paths: &CorePaths,
    config: &ConduitConfig,
    lock: &ConduitLock,
) -> CoreResult<Vec<String>> {
    let mut managed_files: HashSet<String> = HashSet::new();

    for (key, m) in &lock.locked_mods {
        if config.mods.contains_key(key) || m.url != "local" {
            managed_files.insert(m.filename.clone());
        }
    }

    let mut pruned = Vec::new();
    if let Ok(read_dir) = fs::read_dir(paths.mods_dir()) {
        for entry in read_dir {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("jar") {
                continue;
            }
            let filename = match path.file_name().and_then(|n| n.to_str()) {
                Some(f) => f.to_string(),
                None => continue,
            };
            if managed_files.contains(&filename) {
                continue;
            }
            fs::remove_file(&path)?;
            pruned.push(filename);
        }
    }

    Ok(pruned)
}
