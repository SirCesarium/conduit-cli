use dirs::data_local_dir;
use std::fs;
use std::path::{Path, PathBuf};

use crate::core::domain::loader::Loader;

pub struct ConduitPaths;

impl ConduitPaths {
    pub fn get_store_dir() -> PathBuf {
        data_local_dir().map_or_else(|| PathBuf::from(".conduit"), |p| p.join("conduit"))
    }

    pub fn get_manifest_path<P: AsRef<Path>>(project_root: P) -> PathBuf {
        project_root.as_ref().join("conduit.toml")
    }

    pub fn get_lock_path<P: AsRef<Path>>(project_root: P) -> PathBuf {
        project_root.as_ref().join("conduit.lock")
    }

    pub fn ensure_dirs() -> std::io::Result<()> {
        let store = Self::get_store_dir();
        let objects = store.join("objects");

        if !objects.exists() {
            fs::create_dir_all(&objects)?;
        }

        Ok(())
    }

    pub fn get_runtime_id(loader: &Loader, mc_version: &str) -> String {
        let name = match loader {
            Loader::Vanilla => "vanilla".to_string(),
            Loader::Fabric => "fabric".to_string(),
            Loader::Paper => "paper".to_string(),
            Loader::Purpur => "purpur".to_string(),
            Loader::Neoforge { version } => format!("neoforge-{version}"),
            Loader::Forge { version } => format!("forge-{version}"),
        };
        format!("{name}@{mc_version}")
    }

    pub fn is_conduit_file(name: &str) -> bool {
        matches!(
            name,
            "conduit.toml" | "conduit.lock" | ".conduit_runtimes" | ".git" | ".conduit" | "eula.txt"
        )
    }
}
