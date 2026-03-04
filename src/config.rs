use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConduitConfig {
    pub name: String,
    pub mc_version: String,
    pub loader: String,
    pub mods: BTreeMap<String, String>,
    pub settings: Settings,
    pub gui_settings: GuiSettings,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    pub auto_update: bool,
    pub updates_warning: bool,
    pub tunnel: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuiSettings {
    pub auto_wakeup: bool,
    pub remote_panel: RemotePanel,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RemotePanel {
    pub enabled: bool,
    pub port: String,
}

impl Default for ConduitConfig {
    fn default() -> Self {
        Self {
            name: "conduit-server".to_string(),
            mc_version: "1.21.1".to_string(),
            loader: "neoforge@latest".to_string(),
            mods: BTreeMap::new(),
            settings: Settings {
                auto_update: false,
                updates_warning: false,
                tunnel: "playit".to_string(),
            },
            gui_settings: GuiSettings {
                auto_wakeup: true,
                remote_panel: RemotePanel {
                    enabled: true,
                    port: "same".to_string(),
                },
            },
        }
    }
}
