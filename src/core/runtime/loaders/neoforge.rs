use crate::core::{
    events::{CoreCallbacks, CoreEvent},
    installer::download::download_to_path,
};
use regex::Regex;
use std::path::{Path, PathBuf};

pub async fn get_latest_neoforge_version(
    mc_version: &str,
    callbacks: &mut dyn CoreCallbacks,
) -> Result<String, Box<dyn std::error::Error>> {
    callbacks.on_event(CoreEvent::Info(
        "Fetching latest NeoForge version".to_string(),
    ));

    let metadata_url =
        "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml";
    let response = reqwest::get(metadata_url).await?.text().await?;

    let parts: Vec<&str> = mc_version.split('.').collect();

    let major = if parts[0] == "1" {
        parts.get(1).copied().unwrap_or("")
    } else {
        parts[0]
    };

    let minor = if parts[0] == "1" {
        parts.get(2).copied().unwrap_or("0")
    } else {
        parts.get(1).copied().unwrap_or("0")
    };

    let prefix = format!("{major}.{minor}.");

    let pattern = format!(r"<version>({}.*?)</version>", regex::escape(&prefix));
    let re = Regex::new(&pattern)?;

    let versions: Vec<String> = re
        .captures_iter(&response)
        .map(|cap| cap[1].to_string())
        .collect();

    let version = versions
        .last()
        .cloned()
        .ok_or_else(|| format!("No compatible NeoForge version found for MC {mc_version}"))?;

    callbacks.on_event(CoreEvent::Info(format!(
        "Found NeoForge version: {version}"
    )));

    Ok(version)
}

pub async fn download_neoforge_installer(
    mc_version: &str,
    loader_version: &str,
    install_path: &Path,
    callbacks: &mut dyn CoreCallbacks,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let version = if loader_version == "latest" {
        get_latest_neoforge_version(mc_version, callbacks).await?
    } else {
        loader_version.to_string()
    };

    let url = format!(
        "https://maven.neoforged.net/releases/net/neoforged/neoforge/{version}/neoforge-{version}-installer.jar"
    );

    let filename = "neoforge-installer.jar";
    let installer_path = install_path.join(filename);

    let display_name = format!("neoforge-{version}-installer.jar");

    download_to_path(&url, &installer_path, &display_name, callbacks).await?;

    Ok(installer_path)
}

pub async fn execute_neoforge_installer(
    installer_path: &Path,
    callbacks: &mut dyn CoreCallbacks,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    callbacks.on_event(CoreEvent::TaskStarted(format!(
        "Executing NeoForge installer: {}",
        installer_path.display()
    )));

    let status = tokio::process::Command::new("java")
        .arg("-jar")
        .arg(installer_path)
        .arg("--installServer")
        .current_dir(installer_path.parent().unwrap_or(Path::new(".")))
        .stdout(std::process::Stdio::null())
        .status()
        .await?;

    callbacks.on_event(CoreEvent::TaskFinished);

    std::fs::remove_file(installer_path).ok();

    if !status.success() {
        return Err(format!("Installer exited with code: {status}").into());
    }

    Ok(installer_path.to_path_buf())
}

pub fn post_install_neoforge(
    installer_path: &std::path::Path,
    install_path: &Path,
    callbacks: &mut dyn CoreCallbacks,
) -> Result<(), Box<dyn std::error::Error>> {
    callbacks.on_event(CoreEvent::TaskStarted(format!(
        "Executing Post-Install: {}",
        installer_path.display()
    )));

    let eula_path = install_path.join("eula.txt");
    std::fs::write(eula_path, "eula=true")?;

    let log_names = [
        "neoforge-installer.jar.log",
        "installer.log",
        "run.sh",
        "run.bat",
        "user_jvm_args.txt",
    ];

    for log_name in log_names {
        let log_path = install_path.join(log_name);
        if log_path.exists() {
            let _ = std::fs::remove_file(log_path);
        }
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let run_sh = install_path.join("run.sh");
        if let Ok(metadata) = std::fs::metadata(&run_sh) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o755); // rwxr-xr-x
            let _ = std::fs::set_permissions(&run_sh, perms);
        }
    }

    callbacks.on_event(CoreEvent::TaskFinished);

    Ok(())
}
