use crate::core::error::{CoreError, CoreResult};
use crate::core::installer::extra_deps::{ExtraDepsPolicy, InstallerUi};
use crate::core::installer::resolve::{InstallOptions, install_mod};
use crate::core::installer::sync::sync_from_lock;
use crate::core::io::project::lock::{LockedMod, ModSide};
use crate::core::io::project::{ConduitConfig, ConduitLock, ProjectFiles};
use crate::core::apis::modrinth::ModrinthAPI;
use crate::core::mods::local::add_local_mods_to_project;
use crate::core::paths::CorePaths;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

pub struct InstallProjectOptions {
    pub extra_deps_policy: ExtraDepsPolicy,
    pub strict: bool,
    pub force: bool,
    pub allowed_sides: Vec<ModSide>,
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
            allowed_sides: vec![ModSide::Both, ModSide::Client, ModSide::Server],
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
    explicit_side: Option<ModSide>,
) -> CoreResult<()> {
    let mut local_paths = Vec::new();
    let mut local_deps = Vec::new();
    let mut root_modrinth = Vec::new();
    let mut dep_modrinth = Vec::new();

    for input in inputs {
        if let Some(path_str) = input
            .strip_prefix("f:")
            .or_else(|| input.strip_prefix("file:"))
        {
            local_paths.push(PathBuf::from(path_str));
        } else {
            root_modrinth.push(input);
        }
    }

    for dep in explicit_deps {
        if let Some(path_str) = dep.strip_prefix("f:").or_else(|| dep.strip_prefix("file:")) {
            local_deps.push(PathBuf::from(path_str));
        } else {
            dep_modrinth.push(dep);
        }
    }

    if !local_paths.is_empty() {
        add_local_mods_to_project(paths, local_paths, local_deps, explicit_side.as_ref())?;
    }

    if !root_modrinth.is_empty() || !dep_modrinth.is_empty() {
        let mut config = ProjectFiles::load_manifest(paths)?;
        let mut lock = ProjectFiles::load_lock(paths)?;

        for slug in root_modrinth.iter().chain(dep_modrinth.iter()) {
            if let Err(e) = api.get_project(slug).await {
                if let Some(req_err) = e.downcast_ref::<reqwest::Error>()
                    && req_err.status() == Some(reqwest::StatusCode::NOT_FOUND)
                {
                    return Err(CoreError::ProjectNotFound { slug: slug.clone() });
                }

                return Err(e.into());
            }
        }

        for slug in root_modrinth {
            install_mod(
                api,
                paths,
                &slug,
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

        for slug in dep_modrinth {
            install_mod(
                api,
                paths,
                &slug,
                &mut config,
                &mut lock,
                ui,
                InstallOptions {
                    is_root: false,
                    extra_deps_policy: options.extra_deps_policy.clone(),
                },
            )
            .await?;
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

    let allowed = if options.allowed_sides.is_empty() {
        config.instance_type.allowed_sides()
    } else {
        options.allowed_sides.clone()
    };

    if options.force {
        lock = rebuild_lock_from_config(api, paths, ui, &config, &lock, &options).await?;
    }

    let missing_locals: Vec<(String, String)> = lock
        .locked_mods
        .iter()
        .filter(|(_, m)| m.url == "local" && allowed.contains(&m.side))
        .filter(|(_, m)| !paths.mods_dir().join(&m.filename).exists())
        .map(|(slug, m)| (slug.clone(), m.filename.clone()))
        .collect();

    if !missing_locals.is_empty() {
        return Err(CoreError::MissingLocalFiles {
            mods: missing_locals,
        });
    }

    let mods_to_install: Vec<(String, String)> = config
        .mods
        .iter()
        .filter(|(slug, version)| version != &"local" && !lock.locked_mods.contains_key(*slug))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    for (slug, version) in mods_to_install {
        let input = if version == "latest" {
            slug.clone()
        } else {
            format!("{slug}@{version}")
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

    let mods_to_sync = lock
        .locked_mods
        .values()
        .filter(|m| m.url != "local")
        .filter(|m| allowed.contains(&m.side));

    sync_from_lock(paths, mods_to_sync, ui).await?;

    let mut report = SyncProjectReport::default();
    if options.strict {
        report.pruned_files = prune_unmanaged_mods(paths, &config, &lock, &allowed)?;
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
                        side: v.side,
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
    allowed_sides: &[ModSide],
) -> CoreResult<Vec<String>> {
    let mut managed_files: HashSet<String> = HashSet::new();

    for (key, m) in &lock.locked_mods {
        if (config.mods.contains_key(key) || m.url != "local") && allowed_sides.contains(&m.side) {
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
