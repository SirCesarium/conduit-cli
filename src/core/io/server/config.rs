use std::{collections::HashMap, fs, path::Path};

use crate::core::error::{CoreError, CoreResult};
use mc_gate::WakeupCondition as McWakeupCondition;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerConfig {
    pub server: ServerSettings,
    pub spawning: SpawningSettings,
    pub network: NetworkSettings,
    pub access: AccessSettings,
    pub on_demand: OnDemandSettings,
    pub resource_pack: ResourcePackSettings,
    pub performance: PerformanceSettings,
    pub web: WebSettings,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct ServerSettings {
    pub name: String,
    pub level_type: String,
    pub seed: String,
    pub generate_structures: bool,
    pub allow_nether: bool,
    pub hardcore: bool,
    pub difficulty: Difficulty,
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
#[serde(rename_all = "lowercase")]
pub enum Tunnel {
    Playit,
    None,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetworkSettings {
    pub minecraft_port: u16,
    pub max_players: u32,
    pub online_mode: bool,
    pub prevent_proxy_connections: bool,
    pub enforce_secure_profile: bool,
    pub compression_threshold: i32,
    pub tunnel_type: Tunnel,
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
#[serde(rename_all = "lowercase")]
pub enum WakeupConditionMirror {
    Motd,
    Join,
    Disabled,
}

impl From<WakeupConditionMirror> for McWakeupCondition {
    fn from(mirror: WakeupConditionMirror) -> Self {
        match mirror {
            WakeupConditionMirror::Motd => McWakeupCondition::Motd,
            WakeupConditionMirror::Join => McWakeupCondition::Join,
            WakeupConditionMirror::Disabled => McWakeupCondition::Disabled,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OnDemandSettings {
    pub enabled: bool,
    pub proxy_port: u16,
    pub idle_timedout: String,
    pub wakeup_on: WakeupConditionMirror,
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
            .map_err(|e| CoreError::RuntimeError(format!("Error in config.toml: {e}")))?;
        Ok(config)
    }

    pub fn load_or_create<P: AsRef<std::path::Path>>(path: P) -> CoreResult<Self> {
        let path_ref = path.as_ref();

        if !path_ref.exists() {
            let default_config = Self::default();
            default_config.save(path_ref)?;
            return Ok(default_config);
        }

        Self::load(path_ref)
    }

    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> CoreResult<()> {
        let content =
            toml::to_string_pretty(self).map_err(|e| CoreError::RuntimeError(e.to_string()))?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn patch_properties<P: AsRef<Path>>(&self, path: P) -> CoreResult<()> {
        let content = fs::read_to_string(&path)
            .map_err(|_| CoreError::RuntimeError("Could not read server.properties".into()))?;

        let mut updates: HashMap<&str, String> = HashMap::new();

        updates.insert("server-port", self.network.minecraft_port.to_string());
        updates.insert("max-players", self.network.max_players.to_string());
        updates.insert("online-mode", self.network.online_mode.to_string());
        updates.insert("motd", self.network.motd.clone());
        updates.insert(
            "network-compression-threshold",
            self.network.compression_threshold.to_string(),
        );

        updates.insert("level-name", self.server.name.clone());
        updates.insert("level-type", self.server.level_type.clone());
        updates.insert("level-seed", self.server.seed.clone());
        updates.insert(
            "difficulty",
            format!("{:?}", self.server.difficulty).to_lowercase(),
        );
        updates.insert("hardcore", self.server.hardcore.to_string());
        updates.insert("pvp", self.server.pvp.to_string());

        updates.insert("view-distance", self.performance.view_distance.to_string());
        updates.insert(
            "simulation-distance",
            self.performance.simulation_distance.to_string(),
        );
        updates.insert(
            "sync-chunk-writes",
            self.performance.sync_chunk_writes.to_string(),
        );
        updates.insert("spawn-monsters", self.spawning.monsters.to_string());
        updates.insert("spawn-animals", self.spawning.animals.to_string());
        updates.insert("spawn-npcs", self.spawning.npcs.to_string());
        updates.insert(
            "spawn-protection",
            self.spawning.spawn_protection.to_string(),
        );

        updates.insert("white-list", self.access.whitelist.to_string());
        updates.insert("enforce-whitelist", self.access.whitelist.to_string());
        updates.insert(
            "op-permission-level",
            self.access.op_permission_level.to_string(),
        );
        updates.insert(
            "enable-command-block",
            self.access.enable_command_block.to_string(),
        );

        updates.insert(
            "enforce-secure-profile",
            self.network.enforce_secure_profile.to_string(),
        );
        updates.insert(
            "prevent-proxy-connections",
            self.network.prevent_proxy_connections.to_string(),
        );

        updates.insert(
            "entity-broadcast-range-percentage",
            self.performance.entity_range.to_string(),
        );

        let mut new_lines = Vec::new();
        let mut applied_keys = std::collections::HashSet::new();

        for line in content.lines() {
            if line.starts_with('#') || !line.contains('=') {
                new_lines.push(line.to_string());
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, '=').collect();
            let key = parts[0].trim();

            if let Some(new_value) = updates.get(key) {
                new_lines.push(format!("{key}={new_value}"));
                applied_keys.insert(key.to_string());
            } else {
                new_lines.push(line.to_string());
            }
        }

        for (key, value) in updates {
            if !applied_keys.contains(key) {
                new_lines.push(format!("{key}={value}"));
            }
        }

        fs::write(path, new_lines.join("\n"))?;

        Ok(())
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            server: ServerSettings {
                name: "world".into(),
                level_type: "minecraft:\\:normal".into(),
                seed: String::new(),
                generate_structures: true,
                allow_nether: true,
                hardcore: false,
                difficulty: Difficulty::Normal,
                gamemode: "survival".into(),
                force_gamemode: false,
                pvp: true,
            },
            spawning: SpawningSettings {
                monsters: true,
                animals: true,
                npcs: true,
                spawn_protection: 16,
            },
            network: NetworkSettings {
                minecraft_port: 25565,
                max_players: 20,
                online_mode: true,
                prevent_proxy_connections: false,
                enforce_secure_profile: true,
                compression_threshold: 256,
                tunnel_type: Tunnel::None,
                motd: "§bA Minecraft Server §7| §ePowered by Conduit".into(),
            },
            access: AccessSettings {
                whitelist: false,
                op_permission_level: 4,
                function_permission_level: 2,
                player_idle_timeout: 0,
                enable_command_block: false,
            },
            on_demand: OnDemandSettings {
                enabled: false,
                proxy_port: 25567,
                idle_timedout: "10m".into(),
                wakeup_on: WakeupConditionMirror::Motd,
                sleeping_motd: "§c§l⚡ §eStarting server...".into(),
            },
            resource_pack: ResourcePackSettings {
                url: String::new(),
                hash: String::new(),
                required: false,
            },
            performance: PerformanceSettings {
                view_distance: 10,
                simulation_distance: 10,
                entity_range: 100,
                sync_chunk_writes: true,
                max_tick_time: 60000,
                min_ram: "2G".into(),
                max_ram: "4G".into(),
                jvm_args: vec![
                    "-XX:+UseG1GC".into(),
                    "-Dsun.rmi.dgc.server.gcInterval=2147483646".into(),
                ],
            },
            web: WebSettings {
                enabled: false,
                bind: "0.0.0.0:8080".into(),
            },
        }
    }
}
