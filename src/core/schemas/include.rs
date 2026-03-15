use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ConduitInclude {
    pub paths: Vec<String>,
}
