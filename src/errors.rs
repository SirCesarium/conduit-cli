use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConduitError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),

    #[error("disk space or permission error: {0}")]
    Storage(String),

    #[error("network request failed: {0}")]
    Network(#[from] reqwest::Error),

    #[error("failed to download resource from {0}")]
    DownloadFailed(String),

    #[error("api returned an error: {0}")]
    ApiFailure(String),

    #[error("failed to deserialize response: {0}")]
    Deserialize(String),

    #[error("hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },

    #[error("migration failed: {0}")]
    Migration(String),

    #[error("failed to parse configuration: {0}")]
    Config(#[from] toml::de::Error),

    #[error("failed to save configuration: {0}")]
    SaveConfig(#[from] toml::ser::Error),

    #[error("unsupported loader or version: {0}")]
    Unsupported(String),

    #[error("lockfile not found, please install first")]
    NotInstalled,

    #[error("could not find a valid entry point (server.jar or args file)")]
    NoEntryPoint,

    #[error("resource not found: {0}")]
    NotFound(String),
}

pub type ConduitResult<T> = Result<T, ConduitError>;
