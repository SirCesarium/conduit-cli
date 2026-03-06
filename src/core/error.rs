use std::fmt;

#[derive(Debug)]
pub enum CoreError {
    Io(std::io::Error),
    Json(serde_json::Error),
    TomlDe(toml::de::Error),
    TomlSer(toml::ser::Error),
    Reqwest(reqwest::Error),
    MissingConfig,
    MissingLocalDataDir,
    NoCompatibleVersion { slug: String },
    NoFilesForVersion { version: String },
    ProjectNotFound { slug: String },
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreError::Io(e) => write!(f, "IO error: {e}"),
            CoreError::Json(e) => write!(f, "JSON error: {e}"),
            CoreError::TomlDe(e) => write!(f, "TOML decode error: {e}"),
            CoreError::TomlSer(e) => write!(f, "TOML encode error: {e}"),
            CoreError::Reqwest(e) => write!(f, "HTTP error: {e}"),
            CoreError::MissingConfig => write!(f, "No conduit.json found"),
            CoreError::MissingLocalDataDir => write!(f, "Could not find local data directory"),
            CoreError::NoCompatibleVersion { slug } => {
                write!(f, "No compatible version for {slug}")
            }
            CoreError::NoFilesForVersion { version } => {
                write!(f, "No files available for version {version}")
            }
            CoreError::ProjectNotFound { slug } => write!(f, "Project not found: {slug}"),
        }
    }
}

impl std::error::Error for CoreError {}

impl From<std::io::Error> for CoreError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for CoreError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<toml::de::Error> for CoreError {
    fn from(value: toml::de::Error) -> Self {
        Self::TomlDe(value)
    }
}

impl From<toml::ser::Error> for CoreError {
    fn from(value: toml::ser::Error) -> Self {
        Self::TomlSer(value)
    }
}

impl From<reqwest::Error> for CoreError {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(value)
    }
}

pub type CoreResult<T> = Result<T, CoreError>;
