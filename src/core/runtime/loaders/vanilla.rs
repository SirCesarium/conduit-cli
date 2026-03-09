use crate::core::{
    apis::mojang::MojangAPI,
    events::{CoreCallbacks, CoreEvent},
    installer_old::download::download_to_path,
};
use std::path::{Path, PathBuf};

pub async fn get_latest_vanilla_version(
    callbacks: &mut dyn CoreCallbacks,
) -> Result<String, Box<dyn std::error::Error>> {
    callbacks.on_event(CoreEvent::Info(
        "Fetching latest Minecraft Vanilla version".to_string(),
    ));

    let api = MojangAPI::new();
    let manifest = api.get_manifest().await?;
    let version = manifest.latest.release;

    callbacks.on_event(CoreEvent::Info(format!(
        "Found latest Vanilla version: {version}"
    )));

    Ok(version)
}

pub async fn download_vanilla_server(
    mc_version: &str,
    install_path: &Path,
    callbacks: &mut dyn CoreCallbacks,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let api = MojangAPI::new();

    callbacks.on_event(CoreEvent::Info(format!(
        "Resolving Vanilla server for version {mc_version}"
    )));

    let target_version = if mc_version == "latest" {
        let manifest = api.get_manifest().await?;
        manifest.latest.release
    } else {
        mc_version.to_string()
    };

    let details = api.get_version_details(&target_version).await?;
    let server_info = details
        .downloads
        .server
        .ok_or_else(|| format!("No server download found for version {target_version}"))?;

    let jar_path = install_path.join("server.jar");

    download_to_path(
        &server_info.url,
        &jar_path,
        &format!("minecraft-server-{target_version}.jar"),
        callbacks,
    )
    .await?;

    Ok(jar_path)
}

pub fn post_install_vanilla(
    install_path: &Path,
    callbacks: &mut dyn CoreCallbacks,
) -> Result<(), Box<dyn std::error::Error>> {
    callbacks.on_event(CoreEvent::TaskStarted(
        "Executing Vanilla Post-Install".to_string(),
    ));

    let eula_path = install_path.join("eula.txt");
    std::fs::write(eula_path, "eula=true")?;

    let dirs = ["logs", "world"];
    for dir in &dirs {
        let dir_path = install_path.join(dir);
        if !dir_path.exists() {
            let _ = std::fs::create_dir_all(dir_path);
        }
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let jar_path = install_path.join("server.jar");
        if let Ok(metadata) = std::fs::metadata(&jar_path) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o755); // rwxr-xr-x
            let _ = std::fs::set_permissions(&jar_path, perms);
        }
    }

    callbacks.on_event(CoreEvent::TaskFinished);
    Ok(())
}
