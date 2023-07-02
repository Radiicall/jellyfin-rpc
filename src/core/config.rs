use crate::MediaType;

use super::error::ConfigError;
use serde::{Deserialize, Serialize};
use serde_json;
use std::env;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub struct Config {
    pub jellyfin: Jellyfin,
    pub discord: Option<Discord>,
    pub imgur: Option<Imgur>,
    pub images: Option<Images>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Jellyfin {
    pub url: String,
    pub api_key: String,
    pub username: Username,
    pub music: Option<Music>,
    pub blacklist: Option<Blacklist>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Username {
    Vec(Vec<String>),
    String(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Music {
    pub display: Option<Vec<String>>,
    pub separator: Option<char>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Blacklist {
    pub media_types: Option<Vec<String>>,
    pub libraries: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Discord {
    pub application_id: Option<String>,
    pub buttons: Option<Vec<Button>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Button {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Imgur {
    pub client_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Images {
    pub enable_images: Option<bool>,
    pub imgur_images: Option<bool>,
}

pub fn get_config_path() -> Result<String, ConfigError> {
    if cfg!(not(windows)) {
        let user = env::var("USER")?;
        if user != "root" {
            let xdg_config_home = env::var("XDG_CONFIG_HOME")
                .unwrap_or_else(|_| env::var("HOME").unwrap() + "/.config");
            Ok(xdg_config_home + ("/jellyfin-rpc/main.json"))
        } else {
            Ok("/etc/jellyfin-rpc/main.json".to_string())
        }
    } else {
        let app_data = env::var("APPDATA")?;
        Ok(app_data + r"\jellyfin-rpc\main.json")
    }
}

impl Config {
    pub fn load_config(path: String) -> Result<Config, ConfigError> {
        let data = std::fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&data)?;
        Ok(config)
    }
}
