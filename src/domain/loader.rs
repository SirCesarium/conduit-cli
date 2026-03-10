use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "loader_type", rename_all = "snake_case")]
pub enum Loader {
    Vanilla { version: String },
    Neoforge { version: String },
}
