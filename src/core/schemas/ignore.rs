use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConduitIgnore {
    pub rules: Vec<String>,
}

impl Default for ConduitIgnore {
    fn default() -> Self {
        Self {
            rules: vec![
                ".conduit_runtimes/".to_string(),
                "cache/".to_string(),
                "logs/".to_string(),
                "temp/".to_string(),
                ".env/".to_string(),
            ],
        }
    }
}
