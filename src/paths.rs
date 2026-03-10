use dirs::data_local_dir;
use std::fs;
use std::path::{Path, PathBuf};

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
}
