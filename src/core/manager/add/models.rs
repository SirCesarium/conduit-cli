use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ModSide {
    Server,
    Client,
    Both,
}

pub enum RemoteSource {
    Modrinth {
        slug: String,
        version: Option<String>,
    },
}

impl RemoteSource {
    pub fn slug(&self) -> &str {
        match self {
            RemoteSource::Modrinth { slug, .. } => slug,
        }
    }
}

pub enum ResourceSource {
    Remote(RemoteSource),
    Local(PathBuf),
}

pub enum ResourceType {
    Mod,
    Plugin,
    Datapack,
}

pub struct AddRequest {
    pub source: ResourceSource,
    pub side: ModSide,
    pub r#type: ResourceType,
    pub is_dependency: bool,
}
