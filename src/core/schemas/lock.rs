use crate::core::domain::addon::Addon;
use crate::core::domain::loader::Loader;
use crate::core::domain::source::AddonSource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum HashKind {
    Sha1,
    Sha256,
    Sha512,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Lockfile {
    pub version: u32,
    pub instance: InstanceSnapshot,
    pub entries: HashMap<Uuid, LockedAddon>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstanceSnapshot {
    pub minecraft_version: String,
    pub loader: Loader,
    pub loader_hash: Option<String>,
    pub hash_kind: Option<HashKind>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LockedAddon {
    pub metadata: Addon,
    pub source: AddonSource,
}

impl Default for Lockfile {
    fn default() -> Self {
        Self {
            version: 1,
            instance: InstanceSnapshot::default(),
            entries: std::collections::HashMap::new(),
        }
    }
}

impl Default for InstanceSnapshot {
    fn default() -> Self {
        Self {
            minecraft_version: "1.21.11".to_string(),
            loader: Loader::Vanilla,
            loader_hash: None,
            hash_kind: None,
        }
    }
}
