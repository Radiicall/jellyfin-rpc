use super::error::ConfigError;
use crate::prelude::MediaType;
use serde::{Deserialize, Serialize};
use serde_json;
use std::env;

/// Main struct containing every other struct in the file.
///
/// The config file is parsed into this struct.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub struct Config {
    /// Jellyfin configuration.
    ///
    /// Has every required part of the config, hence why its not an `Option<Jellyfin>`.
    pub jellyfin: Jellyfin,
    /// Discord configuration.
    pub discord: Option<Discord>,
    /// Imgur configuration.
    pub imgur: Option<Imgur>,
    /// Images configuration.
    pub images: Option<Images>,
}

/// This struct contains every "required" part of the config.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Jellyfin {
    /// URL to the jellyfin server.
    pub url: String,
    /// Api key from the jellyfin server, used to gather what's being watched.
    pub api_key: String,
    /// Username of the person that info should be gathered from.
    pub username: Username,
    /// Contains configuration for Music display.
    pub music: Option<Music>,
    /// Blacklist configuration.
    pub blacklist: Option<Blacklist>,
    /// Self signed certificate option
    pub self_signed_cert: Option<bool>,
    /// Simple episode name
    pub show_simple: Option<bool>,
    /// Add "0" before season/episode number if lower than 10.
    pub append_prefix: Option<bool>,
    /// Add a divider between numbers
    pub add_divider: Option<bool>
}

/// Username of the person that info should be gathered from.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum Username {
    /// If the username is a `Vec<String>`.
    Vec(Vec<String>),
    /// If the username is a `String`.
    String(String),
}

/// Contains configuration for Music display.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Music {
    /// Display is where you tell the program what should be displayed.
    ///
    /// Example: `vec![String::from("genres"), String::from("year")]`
    pub display: Option<Display>,
    /// Separator is what should be between the artist(s) and the `display` options.
    pub separator: Option<String>,
}

/// Display is where you tell the program what should be displayed.
///
/// Example: `vec![String::from("genres"), String::from("year")]`
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum Display {
    /// If the Display is a `Vec<String>`.
    Vec(Vec<String>),
    /// If the Display is a comma separated `String`.
    String(String),
}

/// Blacklist MediaTypes and libraries.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Blacklist {
    /// `Vec<String>` of MediaTypes to blacklist
    pub media_types: Option<Vec<MediaType>>,
    /// `Vec<String>` of libraries to blacklist
    pub libraries: Option<Vec<String>>,
}

/// Discord configuration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Discord {
    /// Set a custom Application ID to be used.
    pub application_id: Option<String>,
    /// Set custom buttons to be displayed.
    pub buttons: Option<Vec<Button>>,
    /// Show status when media is paused
    pub show_paused: Option<bool>,
}

/// Button struct
///
/// Contains information about buttons
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Button {
    /// What the name should be showed as in Discord.
    pub name: String,
    /// What clicking it should point to in Discord.
    pub url: String,
}

impl Default for Button {
    fn default() -> Self {
        Self {
            name: String::from("dynamic"),
            url: String::from("dynamic"),
        }
    }
}

/// Imgur configuration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Imgur {
    /// Contains the client ID used to upload images to imgur.
    pub client_id: Option<String>,
}

/// Images configuration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Images {
    /// Enables images, not everyone wants them so its a toggle.
    pub enable_images: Option<bool>,
    /// Enables imgur images.
    pub imgur_images: Option<bool>,
}

/// Find config.json in filesystem.
///
/// This is to avoid the user having to specify a filepath on launch.
///
/// Default config path depends on OS
/// Windows: `%appdata%\jellyfin-rpc\config.json`
/// Linux/macOS: `~/.config/jellyfin-rpc/config.json`
pub fn get_config_path() -> Result<String, ConfigError> {
    if cfg!(not(windows)) {
        let xdg_config_home = match env::var("XDG_CONFIG_HOME") {
            Ok(xdg_config_home) => xdg_config_home,
            Err(_) => env::var("HOME")? + "/.config",
        };

        Ok(xdg_config_home + ("/jellyfin-rpc/main.json"))
    } else {
        let app_data = env::var("APPDATA")?;
        Ok(app_data + r"\jellyfin-rpc\main.json")
    }
}

impl Config {
    /// Loads the config from the given path.
    pub fn load(path: &str) -> Result<Config, ConfigError> {
        let data = std::fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&data)?;
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            jellyfin: Jellyfin {
                url: "".to_string(),
                username: Username::String("".to_string()),
                api_key: "".to_string(),
                music: None,
                blacklist: None,
                self_signed_cert: None,
                show_simple: Some(false),
                append_prefix: Some(false),
                add_divider: Some(false)
            },
            discord: None,
            imgur: None,
            images: None,
        }
    }
}
