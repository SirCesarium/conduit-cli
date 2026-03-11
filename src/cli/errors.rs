#![allow(unused_assignments, dead_code)]
use conduit_cli::errors::ConduitError;
use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
#[error("Conduit Error")]
pub struct CliError {
    #[source]
    pub inner: ConduitError,

    #[diagnostic(code)]
    pub code: &'static str,

    #[help]
    pub help: Option<String>,
}

impl From<ConduitError> for CliError {
    fn from(inner: ConduitError) -> Self {
        let (code, help) = match &inner {
            ConduitError::AlreadyInitialized(_) => (
                "conduit::project::exists",
                Some("Remove the 'conduit.toml' file if you want to start over, or edit it directly.".to_string())
            ),
            ConduitError::NotInstalled => (
                "conduit::project::not_installed",
                Some("Run 'conduit install' to download the server and mods.".to_string())
            ),
            ConduitError::NoEntryPoint => (
                "conduit::runtime::missing_jar",
                Some("The server JAR wasn't found. This can happen if the installation was interrupted.".to_string())
            ),
            ConduitError::Io(e) => (
                "conduit::system::io",
                Some(format!("Check if the path is accessible. Original error: {e}"))
            ),
            ConduitError::Network(_) => (
                "conduit::network::request_failed",
                Some("Check your internet connection or Modrinth/Mojang API status.".to_string())
            ),
            ConduitError::HashMismatch { .. } => (
                "conduit::security::hash_mismatch",
                Some("The downloaded file is corrupted. Try clearing the cache and running the command again.".to_string())
            ),
            _ => ("conduit::internal::error", None),
        };

        Self { inner, code, help }
    }
}
