use crate::errors::{ConduitError, ConduitResult};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use zip::write::FileOptions;
use zip::{ZipArchive, ZipWriter};

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

    pub fn create<P: AsRef<Path>>(path: P) -> ConduitResult<ZipWriter<File>> {
        let file = File::create(&path)?;
        Ok(ZipWriter::new(file))
    }

    pub fn read_metadata(archive: &mut ZipArchive<File>, name: &str) -> ConduitResult<String> {
        let mut content = String::new();
        let mut file = Self::get_validated_file(archive, name, 25 * 1024 * 1024)?;
        file.read_to_string(&mut content).map_err(|e| {
            ConduitError::Io(std::io::Error::other(format!(
                "Failed to read metadata '{name}': {e}"
            )))
        })?;
        Ok(content)
    }

    pub fn read_bytes(archive: &mut ZipArchive<File>, name: &str) -> ConduitResult<Vec<u8>> {
        let mut buffer = Vec::new();
        let mut file = Self::get_validated_file(archive, name, 100 * 1024 * 1024)?;
        file.read_to_end(&mut buffer).map_err(|e| {
            ConduitError::Io(std::io::Error::other(format!(
                "Failed to read bytes from '{name}': {e}"
            )))
        })?;
        Ok(buffer)
    }

    pub fn add_file<W: Write + std::io::Seek>(
        writer: &mut ZipWriter<W>,
        name: &str,
        content: &[u8],
    ) -> ConduitResult<()> {
        let options: FileOptions<()> = FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o644);

        writer.start_file(name, options).map_err(|e| {
            ConduitError::Storage(format!("Failed to create entry '{name}' in archive: {e}"))
        })?;
        writer.write_all(content)?;
        Ok(())
    }

    fn get_validated_file<'a>(
        archive: &'a mut ZipArchive<File>,
        name: &str,
        size_limit: u64,
    ) -> ConduitResult<zip::read::ZipFile<'a, File>> {
        let normalized_name = name.replace('\\', "/");

        if normalized_name.contains("..") || normalized_name.starts_with('/') {
            return Err(ConduitError::Storage(format!(
                "Security violation: malicious path detected: {name}"
            )));
        }

        let file = archive
            .by_name(name)
            .map_err(|_| ConduitError::NotFound(format!("Entry '{name}' not found in archive")))?;

        if file.size() > size_limit {
            return Err(ConduitError::Storage(format!(
                "Security violation: entry '{name}' size ({} MB) exceeds limit ({} MB)",
                file.size() / 1024 / 1024,
                size_limit / 1024 / 1024
            )));
        }

        Ok(file)
    }

    pub fn read_and_deserialize<T>(archive: &mut ZipArchive<File>, name: &str) -> ConduitResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let raw = Self::read_metadata(archive, name)?;

        let extension = Path::new(name)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(str::to_lowercase);

        match extension.as_deref() {
            Some("json") => serde_json::from_str(&raw)
                .map_err(|e| ConduitError::Parsing(format!("JSON error in {name}: {e}"))),
            Some("toml") => toml::from_str(&raw)
                .map_err(|e| ConduitError::Parsing(format!("TOML error in {name}: {e}"))),
            _ => Err(ConduitError::Parsing(format!(
                "Unsupported or missing file extension for deserialization: {name}"
            ))),
        }
    }

    pub fn serialize_and_add<T, W>(
        writer: &mut zip::ZipWriter<W>,
        name: &str,
        data: &T,
    ) -> ConduitResult<()>
    where
        T: serde::Serialize,
        W: std::io::Write + std::io::Seek,
    {
        let extension = std::path::Path::new(name)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(str::to_lowercase);

        let bytes = match extension.as_deref() {
            Some("json") => serde_json::to_vec(data)
                .map_err(|e| ConduitError::Parsing(format!("JSON serialize error: {e}")))?,
            Some("toml" | "lock") => toml::to_string(data)
                .map_err(|e| ConduitError::Parsing(format!("TOML serialize error: {e}")))?
                .into_bytes(),
            _ => {
                return Err(ConduitError::Parsing(
                    "Unsupported export format".to_string(),
                ));
            }
        };

        Self::add_file(writer, name, &bytes)
    }
}
