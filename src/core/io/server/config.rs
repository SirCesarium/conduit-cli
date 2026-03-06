use serde::{Deserialize, Serialize};
use crate::core::error::{CoreError, CoreResult};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerConfig {
    pub server: ServerSettings,
    pub spawning: SpawningSettings,
    pub network: NetworkSettings,
    pub access: AccessSettings,
    pub gateway: GatewaySettings,
    pub resource_pack: ResourcePackSettings,
    pub performance: PerformanceSettings,
    pub web: WebSettings,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerSettings {
    pub name: String,
    pub level_type: String,
    pub seed: String,
    pub generate_structures: bool,
    pub allow_nether: bool,
    pub hardcore: bool,
    pub difficulty: String,
    pub gamemode: String,
    pub force_gamemode: bool,
    pub pvp: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpawningSettings {
    pub monsters: bool,
    pub animals: bool,
    pub npcs: bool,
    pub spawn_protection: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetworkSettings {
    pub port: u16,
    pub internal_port: u16,
    pub max_players: u32,
    pub online_mode: bool,
    pub prevent_proxy_connections: bool,
    pub enforce_secure_profile: bool,
    pub compression_threshold: i32,
    pub tunnel_type: String,
    pub motd: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccessSettings {
    pub whitelist: bool,
    pub op_permission_level: u8,
    pub function_permission_level: u8,
    pub player_idle_timeout: u32,
    pub enable_command_block: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GatewaySettings {
    pub enabled: bool,
    pub auto_sleep: bool,
    pub auto_wakeup: bool,
    pub trigger: String,
    pub sleeping_motd: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResourcePackSettings {
    pub url: String,
    pub hash: String,
    pub required: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PerformanceSettings {
    pub view_distance: u32,
    pub simulation_distance: u32,
    pub entity_range: u32,
    pub sync_chunk_writes: bool,
    pub max_tick_time: u64,
    pub min_ram: String,
    pub max_ram: String,
    pub jvm_args: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebSettings {
    pub enabled: bool,
    pub bind: String,
}

impl ServerConfig {
    pub fn load<P: AsRef<std::path::Path>>(path: P) -> CoreResult<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|_| CoreError::RuntimeError("Could not find config.toml".into()))?;
        let config: ServerConfig = toml::from_str(&content)
            .map_err(|e| CoreError::RuntimeError(format!("Error in config.toml: {}", e)))?;
        Ok(config)
    }

    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> CoreResult<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| CoreError::RuntimeError(e.to_string()))?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
