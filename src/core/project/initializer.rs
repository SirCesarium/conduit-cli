use crate::core::error::CoreResult;
use crate::core::io::project::{ConduitConfig, InstanceType, ProjectFiles};
use crate::core::io::server::config::ServerConfig;
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
    if let Some(t) = params.instance_type.clone() {
        config.instance_type = t;
    }
    if let Some(v) = params.mc_version {
        config.mc_version = v;
    }
    if let Some(l) = params.loader {
        config.loader = l;
    }

    ProjectFiles::save_manifest(paths, &config)?;

    if let Some(InstanceType::Server) = params.instance_type {
        let default_config = ServerConfig::default();
        default_config.save(paths.config_path())?;
    }

    Ok(config)
}
