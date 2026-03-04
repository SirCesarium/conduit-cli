use crate::core::error::CoreResult;
use crate::core::events::{CoreCallbacks, CoreEvent};
use crate::core::installer::download::download_to_path;
use crate::core::paths::CorePaths;
use crate::lock::LockedMod;
use std::path::Path;

pub async fn sync_from_lock(
    paths: &CorePaths,
    locked_mods: impl IntoIterator<Item = &'_ LockedMod>,
    callbacks: &mut dyn CoreCallbacks,
) -> CoreResult<()> {
    crate::core::installer::resolve::ensure_dirs(paths)?;

    for locked_mod in locked_mods {
        install_from_lock(locked_mod, paths.cache_dir(), paths.mods_dir(), callbacks).await?;
    }

    Ok(())
}

async fn install_from_lock(
    mod_data: &LockedMod,
    cache_dir: &Path,
    mods_dir: &Path,
    callbacks: &mut dyn CoreCallbacks,
) -> CoreResult<()> {
    let cached_path = cache_dir.join(format!("{}.jar", mod_data.hash));
    let dest_path = mods_dir.join(&mod_data.filename);

    if dest_path.exists() {
        return Ok(());
    }

    if !cached_path.exists() {
        download_to_path(&mod_data.url, &cached_path, &mod_data.filename, callbacks).await?;
    }

    crate::core::installer::resolve::hard_link_jar(&cached_path, &dest_path)?;

    callbacks.on_event(CoreEvent::LinkedFile {
        filename: mod_data.filename.clone(),
    });

    Ok(())
}
