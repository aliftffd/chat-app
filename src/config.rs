use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::Context;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub client: ClientConfig,
    pub display: DisplayConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    #[serde(default = "default_server_address")]
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClientConfig {
    #[serde(default)]
    pub auto_reconnect: bool,
    pub device_id: Option<String>,
    pub username: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DisplayConfig {
    #[serde(default = "default_true")]
    pub show_timestamps: bool,
    #[serde(default = "default_true")]
    pub show_colors: bool,
}

fn default_server_address() -> String {
    "127.0.0.1:8080".to_string()
}

fn default_true() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                address: default_server_address(),
            },
            client: ClientConfig {
                auto_reconnect: false,
                device_id: None,
                username: None,
            },
            display: DisplayConfig {
                show_timestamps: true,
                show_colors: true,
            },
        }
    }
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let config_path = Self::config_path()?;
        
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)
                .context("Failed to read config file")?;
            let config: Config = toml::from_str(&content)
                .context("Failed to parse config file")?;
            Ok(config)
        } else {
            // Create default config
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }
    
    pub fn save(&self) -> anyhow::Result<()> {
        let config_path = Self::config_path()?;
        
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }
        
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        std::fs::write(&config_path, content)
            .context("Failed to write config file")?;
        
        println!("Config saved to: {}", config_path.display());
        Ok(())
    }
    
    pub fn config_path() -> anyhow::Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        Ok(config_dir.join("chat-app").join("config.toml"))
    }
}
