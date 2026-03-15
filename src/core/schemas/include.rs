use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConduitInclude {
    pub paths: Vec<String>,
}

impl Default for ConduitInclude {
    fn default() -> Self {
        Self {
            paths: vec!["config/".to_string()],
        }
    }
}
