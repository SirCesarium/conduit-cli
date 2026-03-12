use crate::errors::{ConduitError, ConduitResult};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

pub struct SafeArchive;

impl SafeArchive {
    pub fn open<P: AsRef<Path>>(path: P) -> ConduitResult<ZipArchive<File>> {
        let file = File::open(&path)?;
        ZipArchive::new(file).map_err(|e| {
            ConduitError::Storage(format!(
                "Failed to open archive '{}': {}",
                path.as_ref().display(),
                e
            ))
        })
    }

    pub fn read_file(archive: &mut ZipArchive<File>, name: &str) -> ConduitResult<String> {
        if name.contains("..") || name.starts_with('/') || name.contains('\\') {
            return Err(ConduitError::Storage(format!(
                "Security violation: malicious path detected in archive entry: {name}"
            )));
        }

        let mut file = archive
            .by_name(name)
            .map_err(|_| ConduitError::NotFound(format!("Entry '{name}' not found in archive")))?;

        if file.size() > 10 * 1024 * 1024 {
            return Err(ConduitError::Storage(format!(
                "Security violation: entry '{name}' exceeds size limit (10MB)"
            )));
        }

        let mut content = String::new();
        file.read_to_string(&mut content).map_err(|e| {
            ConduitError::Io(std::io::Error::other(format!(
                "Failed to read entry '{name}': {e}"
            )))
        })?;

        Ok(content)
    }
}
