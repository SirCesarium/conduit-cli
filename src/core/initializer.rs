use crate::core::error::CoreResult;
use crate::core::filesystem::config::{ConduitConfig, InstanceType};
use crate::core::filesystem::lock::ConduitLock;
use crate::core::paths::CorePaths;

pub struct InitParams {
    pub name: Option<String>,
    pub instance_type: Option<InstanceType>,
    pub mc_version: Option<String>,
    pub loader: Option<String>,
}

pub fn init_project(paths: &CorePaths, params: InitParams) -> CoreResult<ConduitConfig> {
    let mut config = ConduitConfig::default();

    if let Some(n) = params.name {
        config.name = n;
    }
    if let Some(t) = params.instance_type {
        config.instance_type = t;
    }
    if let Some(v) = params.mc_version {
        config.mc_version = v;
    }
    if let Some(l) = params.loader {
        config.loader = l;
    }

    ConduitLock::save_config(paths, &config)?;
    Ok(config)
}
