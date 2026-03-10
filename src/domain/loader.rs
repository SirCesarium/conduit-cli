use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "loader_type", rename_all = "snake_case")]
pub enum Loader {
    Vanilla,
    Neoforge { version: String },
    Fabric,
    Forge { version: String },
    Paper,
    Purpur,
}
