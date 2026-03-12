use serde::{Deserialize, Serialize};
use strum::Display;

use crate::errors::ConduitError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Display)]
#[serde(tag = "loader_type", rename_all = "snake_case")]
pub enum Loader {
    Vanilla,
    Neoforge { version: String },
    Fabric,
    Forge { version: String },
    Paper,
    Purpur,
}

impl Loader {
    pub fn pretty_name(&self) -> String {
        match self {
            Loader::Vanilla => "Vanilla".to_string(),
            Loader::Fabric => "Fabric".to_string(),
            Loader::Paper => "Paper".to_string(),
            Loader::Purpur => "Purpur".to_string(),
            Loader::Neoforge { version } => format!("NeoForge ({version})"),
            Loader::Forge { version } => format!("Forge ({version})"),
        }
    }

    pub fn from_string(name: &str, version: Option<&str>) -> Result<Self, ConduitError> {
        let name_lower = name.to_lowercase();

        match name_lower.as_str() {
            "vanilla" => Ok(Loader::Vanilla),
            "fabric" => Ok(Loader::Fabric),
            "paper" => Ok(Loader::Paper),
            "purpur" => Ok(Loader::Purpur),
            "neoforge" | "forge" => {
                let v = version.ok_or_else(|| {
                    ConduitError::Validation(format!(
                        "Loader {name} requires a version (e.g. {name}@version)"
                    ))
                })?;

                if name_lower == "neoforge" {
                    Ok(Loader::Neoforge {
                        version: v.to_string(),
                    })
                } else {
                    Ok(Loader::Forge {
                        version: v.to_string(),
                    })
                }
            }
            _ => Err(ConduitError::Unsupported(name.to_string())),
        }
    }
}
