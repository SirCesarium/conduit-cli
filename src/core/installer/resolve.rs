use crate::config::ConduitConfig;
use crate::core::error::{CoreError, CoreResult};
use crate::core::events::CoreEvent;
use crate::core::installer::download::download_to_path;
use crate::core::installer::extra_deps::{
    ExtraDepCandidate, ExtraDepDecision, ExtraDepRequest, ExtraDepsPolicy, InstallerUi,
};
use crate::core::paths::CorePaths;
use crate::inspector::JarInspector;
use crate::lock::{ConduitLock, LockedMod};
use crate::modrinth::ModrinthAPI;
use async_recursion::async_recursion;
use std::fs;
use std::path::{Path, PathBuf};

pub struct InstallOptions {
    pub is_root: bool,
    pub extra_deps_policy: ExtraDepsPolicy,
}

impl Default for InstallOptions {
    fn default() -> Self {
        Self {
            is_root: true,
            extra_deps_policy: ExtraDepsPolicy::Skip,
        }
    }
}

pub async fn install_mod(
    api: &ModrinthAPI,
    paths: &CorePaths,
    input: &str,
    config: &mut ConduitConfig,
    lock: &mut ConduitLock,
    ui: &mut dyn InstallerUi,
    options: InstallOptions,
) -> CoreResult<()> {
    fs::create_dir_all(paths.cache_dir())?;
    fs::create_dir_all(paths.mods_dir())?;

    install_recursive(
        api,
        paths,
        input,
        config,
        lock,
        ui,
        options.is_root,
        options.extra_deps_policy,
    )
    .await
}

#[async_recursion(?Send)]
async fn install_recursive(
    api: &ModrinthAPI,
    paths: &CorePaths,
    input: &str,
    config: &mut ConduitConfig,
    lock: &mut ConduitLock,
    ui: &mut dyn InstallerUi,
    is_root: bool,
    extra_deps_policy: ExtraDepsPolicy,
) -> CoreResult<()> {
    let parts: Vec<&str> = input.split('@').collect();
    let slug_or_id = parts[0];
    let requested_version = parts.get(1).copied();

    let project = api.get_project(slug_or_id).await?;
    let current_slug = project.slug;

    if is_root && config.mods.contains_key(&current_slug) {
        ui.on_event(CoreEvent::AlreadyInstalled {
            slug: current_slug,
        });
        return Ok(());
    }

    if !is_root && lock.locked_mods.contains_key(&current_slug) {
        return Ok(());
    }

    if is_root && lock.locked_mods.contains_key(&current_slug) {
        config
            .mods
            .insert(current_slug.clone(), "latest".to_string());
        ui.on_event(CoreEvent::AddedAsDependency {
            slug: current_slug,
        });
        return Ok(());
    }

    ui.on_event(CoreEvent::Info(format!(
        "Installing {}",
        project.title
    )));

    let loader_filter = config.loader.split('@').next().unwrap_or("fabric");

    let versions = api
        .get_versions(&current_slug, Some(loader_filter), Some(&config.mc_version))
        .await?;

    let selected_version = if let Some(req) = requested_version {
        versions
            .iter()
            .find(|v| v.version_number == req || v.id == req)
            .or_else(|| versions.first())
    } else {
        versions.first()
    }
    .ok_or_else(|| CoreError::NoCompatibleVersion { slug: current_slug.clone() })?;

    let file = selected_version
        .files
        .iter()
        .find(|f| f.primary)
        .or(selected_version.files.first())
        .ok_or_else(|| CoreError::NoFilesForVersion {
            version: selected_version.version_number.clone(),
        })?;

    let sha1 = file.hashes.get("sha1").cloned().unwrap_or_default();

    let cached_path = paths.cache_dir().join(format!("{}.jar", sha1));
    let dest_path = paths.mods_dir().join(&file.filename);

    if !cached_path.exists() {
        download_to_path(&file.url, &cached_path, &file.filename, ui).await?;
    }

    if dest_path.exists() {
        fs::remove_file(&dest_path)?;
    }
    fs::hard_link(&cached_path, &dest_path)?;

    if is_root {
        config
            .mods
            .insert(current_slug.clone(), selected_version.version_number.clone());
    }

    let mut current_deps = Vec::new();
    for dep in &selected_version.dependencies {
        if dep.dependency_type == "required" {
            if let Some(proj_id) = &dep.project_id {
                current_deps.push(proj_id.clone());
            }
        }
    }

    lock.locked_mods.insert(
        current_slug.clone(),
        LockedMod {
            id: selected_version.project_id.clone(),
            version_id: selected_version.id.clone(),
            filename: file.filename.clone(),
            url: file.url.clone(),
            hash: sha1,
            dependencies: current_deps.clone(),
        },
    );

    for dep_id in current_deps {
        install_recursive(
            api,
            paths,
            &dep_id,
            config,
            lock,
            ui,
            false,
            extra_deps_policy.clone(),
        )
        .await?;
    }

    if let ExtraDepsPolicy::Skip = extra_deps_policy {
    } else {
        crawl_extra_dependencies(
            api,
            paths,
            &dest_path,
            config,
            lock,
            &current_slug,
            ui,
            extra_deps_policy,
        )
        .await?;
    }

    ui.on_event(CoreEvent::Installed {
        slug: current_slug,
        title: project.title,
    });

    Ok(())
}

async fn crawl_extra_dependencies(
    api: &ModrinthAPI,
    paths: &CorePaths,
    jar_path: &PathBuf,
    config: &mut ConduitConfig,
    lock: &mut ConduitLock,
    parent_slug: &str,
    ui: &mut dyn InstallerUi,
    extra_deps_policy: ExtraDepsPolicy,
) -> CoreResult<()> {
    let internal_deps = match JarInspector::inspect_neoforge(jar_path) {
        Ok(deps) => deps,
        Err(_) => return Ok(()),
    };

    let loader_filter = config
        .loader
        .split('@')
        .next()
        .unwrap_or("neoforge")
        .to_string();
    let mc_version = config.mc_version.clone();

    for tech_id in internal_deps {
        let is_installed = lock.locked_mods.values().any(|m| m.id == tech_id)
            || lock.locked_mods.contains_key(&tech_id)
            || config.mods.contains_key(&tech_id);

        if is_installed {
            continue;
        }

        let facets = format!(
            "[[\"categories:{}\"],[\"versions:{}\"]]",
            loader_filter, mc_version
        );
        let search_results = api
            .search(&tech_id, 5, 0, "relevance", Some(facets))
            .await?;

        let mut candidates: Vec<ExtraDepCandidate> = Vec::new();

        let mut exact_match_slug = None;
        if let Ok(exact) = api.get_project(&tech_id).await {
            exact_match_slug = Some(exact.slug.clone());
            candidates.push(ExtraDepCandidate {
                title: exact.title,
                slug: exact.slug,
                is_exact_match: true,
            });
        }

        for hit in &search_results.hits {
            if Some(&hit.slug) != exact_match_slug.as_ref() {
                candidates.push(ExtraDepCandidate {
                    title: hit.title.clone(),
                    slug: hit.slug.clone(),
                    is_exact_match: false,
                });
            }
        }

        if candidates.is_empty() {
            continue;
        }

        let parent_filename = jar_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown file")
            .to_string();

        let decision = match extra_deps_policy {
            ExtraDepsPolicy::Skip => ExtraDepDecision::Skip,
            ExtraDepsPolicy::AutoExactMatch => exact_match_slug
                .clone()
                .map(ExtraDepDecision::InstallSlug)
                .unwrap_or(ExtraDepDecision::Skip),
            ExtraDepsPolicy::Callback => ui.choose_extra_dep(ExtraDepRequest {
                tech_id: tech_id.clone(),
                parent_slug: parent_slug.to_string(),
                parent_filename,
                candidates,
            }),
        };

        let slug_to_install = match decision {
            ExtraDepDecision::Skip => continue,
            ExtraDepDecision::InstallSlug(s) => s,
        };

        ui.on_event(CoreEvent::Info(format!(
            "Installing extra dependency {slug_to_install}"
        )));

        install_recursive(
            api,
            paths,
            &slug_to_install,
            config,
            lock,
            ui,
            false,
            extra_deps_policy.clone(),
        )
        .await?;

        if let Some(installed_mod) = lock.locked_mods.get(&slug_to_install) {
            let installed_id = installed_mod.id.clone();
            if let Some(parent) = lock.locked_mods.get_mut(parent_slug) {
                if !parent.dependencies.contains(&installed_id) {
                    parent.dependencies.push(installed_id);
                }
            }
        }
    }

    Ok(())
}

pub fn ensure_dirs(paths: &CorePaths) -> CoreResult<()> {
    fs::create_dir_all(paths.cache_dir())?;
    fs::create_dir_all(paths.mods_dir())?;
    Ok(())
}

pub fn hard_link_jar(cache_path: &Path, dest_path: &Path) -> CoreResult<()> {
    if dest_path.exists() {
        fs::remove_file(dest_path)?;
    }
    fs::hard_link(cache_path, dest_path)?;
    Ok(())
}
