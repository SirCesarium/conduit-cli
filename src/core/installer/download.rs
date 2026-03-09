use crate::core::context::ConduitContext;
use crate::core::error::CoreResult;
use std::fs;
use std::path::Path;

pub async fn download_and_link(
    ctx: &ConduitContext,
    url: &str,
    filename: &str,
    hash: &str,
) -> CoreResult<()> {
    let cache_dir = ctx.paths.cache_dir();
    let mods_dir = ctx.paths.mods_dir();

    if !cache_dir.exists() {
        fs::create_dir_all(cache_dir)?;
    }
    if !mods_dir.exists() {
        fs::create_dir_all(mods_dir)?;
    }

    let cached_path = cache_dir.join(format!("{hash}.jar"));
    let dest_path = mods_dir.join(filename);

    if !cached_path.exists() {
        download_to_cache(ctx, url, &cached_path).await?;
    }

    if dest_path.exists() {
        fs::remove_file(&dest_path)?;
    }

    fs::hard_link(&cached_path, &dest_path)?;

    Ok(())
}

async fn download_to_cache(
    ctx: &ConduitContext,
    url: &str,
    dest: &Path,
) -> CoreResult<()> {
    let response = ctx.api.client
        .get(url)
        .send()
        .await?
        .bytes()
        .await?;

    fs::write(dest, response)?;

    Ok(())
}
