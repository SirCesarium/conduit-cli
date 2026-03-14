use dirs::data_local_dir;
use std::fs;
use std::path::{Path, PathBuf};

use crate::core::domain::loader::Loader;

#[derive(Clone, Debug)]
pub struct ConduitPaths {
    pub root: PathBuf,
    pub store: PathBuf,
}

impl ConduitPaths {
    pub fn new<P: AsRef<Path>>(project_root: P) -> Self {
        let root = project_root.as_ref().to_path_buf();
        let store =
            data_local_dir().map_or_else(|| root.join(".conduit_data"), |p| p.join("conduit"));

        Self { root, store }
    }

    pub fn manifest_name() -> &'static str {
        "conduit.toml"
    }

    pub fn lockfike_name() -> &'static str {
        "conduit.lock"
    }

    pub fn manifest(&self) -> PathBuf {
        self.root.join(Self::manifest_name())
    }

    pub fn lock(&self) -> PathBuf {
        self.root.join(Self::lockfike_name())
    }

    pub fn runtimes_dir(&self) -> PathBuf {
        self.store.join("runtimes")
    }

    pub fn objects_dir(&self) -> PathBuf {
        self.store.join("objects")
    }

    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        fs::create_dir_all(self.objects_dir())?;
        fs::create_dir_all(self.runtimes_dir())?;
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
            "conduit.toml"
                | "conduit.lock"
                | ".conduit_runtimes"
                | ".git"
                | ".conduit"
                | "eula.txt"
        )
    }
}
