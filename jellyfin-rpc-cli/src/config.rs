use jellyfin_rpc::{Button, MediaType};
use log::debug;
use serde::{Deserialize, Serialize};
use std::env;

/// Main struct containing every other struct in the file.
///
/// The config file is parsed into this struct.
pub struct Config {
    /// Jellyfin configuration.
    ///
    /// Has every required part of the config, hence why its not an `Option<Jellyfin>`.
    pub jellyfin: Jellyfin,
    /// Discord configuration.
    pub discord: Discord,
    /// Imgur configuration.
    pub imgur: Imgur,
    /// Images configuration.
    pub images: Images,
}

/// This struct contains every "required" part of the config.
pub struct Jellyfin {
    /// URL to the jellyfin server.
    pub url: String,
    /// Api key from the jellyfin server, used to gather what's being watched.
    pub api_key: String,
    /// Username of the person that info should be gathered from.
    pub username: Vec<String>,
    /// Contains configuration for Music display.
    pub music: DisplayOptions,
    /// Contains configuration for Movie display.
    pub movies: DisplayOptions,
    /// Blacklist configuration.
    pub blacklist: Blacklist,
    /// Self signed certificate option
    pub self_signed_cert: bool,
    /// Simple episode name
    pub show_simple: bool,
    /// Add "0" before season/episode number if lower than 10.
    pub append_prefix: bool,
    /// Add a divider between numbers
    pub add_divider: bool,
}

/// Contains configuration for Music/Movie display.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DisplayOptions {
    pub state_text: Option<String>,
    pub details_text: Option<String>,
    pub image_text: Option<String>,
}

/// Discord configuration
pub struct Discord {
    /// Set a custom Application ID to be used.
    pub application_id: Option<String>,
    /// Set custom buttons to be displayed.
    pub buttons: Option<Vec<Button>>,
    /// Show status when media is paused
    pub show_paused: bool,
}

/// Images configuration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Images {
    /// Enables images, not everyone wants them so its a toggle.
    pub enable_images: bool,
    /// Enables imgur images.
    pub imgur_images: bool,
}

impl Config {
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub struct ConfigBuilder {
    pub jellyfin: JellyfinBuilder,
    pub discord: Option<DiscordBuilder>,
    pub imgur: Option<Imgur>,
    pub images: Option<ImagesBuilder>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct JellyfinBuilder {
    pub url: String,
    pub api_key: String,
    pub username: Username,
    pub music: Option<DisplayOptions>,
    pub movies: Option<DisplayOptions>,
    pub blacklist: Option<Blacklist>,
    pub self_signed_cert: Option<bool>,
    pub show_simple: Option<bool>,
    pub append_prefix: Option<bool>,
    pub add_divider: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum Username {
    /// If the username is a `Vec<String>`.
    Vec(Vec<String>),
    /// If the username is a `String`.
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DiscordBuilder {
    pub application_id: Option<String>,
    pub buttons: Option<Vec<Button>>,
    pub show_paused: Option<bool>,
}

/// Imgur configuration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Imgur {
    /// Contains the client ID used to upload images to imgur.
    pub client_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ImagesBuilder {
    pub enable_images: Option<bool>,
    pub imgur_images: Option<bool>,
}

/// Find urls.json in filesystem, used to store images that were already previously uploaded to imgur.
///
/// This is to avoid the user having to specify a filepath on launch.
///
/// Default urls.json path depends on OS
/// Windows: `%appdata%\jellyfin-rpc\urls.json`
/// Linux/macOS: `~/.config/jellyfin-rpc/urls.json`
pub fn get_urls_path() -> Result<String, Box<dyn std::error::Error>> {
    if cfg!(not(windows)) {
        debug!("Platform is not Windows");
        let xdg_config_home = match env::var("XDG_CONFIG_HOME") {
            Ok(xdg_config_home) => xdg_config_home,
            Err(_) => env::var("HOME")? + "/.config",
        };

        Ok(xdg_config_home + ("/jellyfin-rpc/urls.json"))
    } else {
        debug!("Platform is Windows");
        let app_data = env::var("APPDATA")?;
        Ok(app_data + r"\jellyfin-rpc\urls.json")
    }
}

/// Find config.json in filesystem.
///
/// This is to avoid the user having to specify a filepath on launch.
///
/// Default config path depends on OS
/// Windows: `%appdata%\jellyfin-rpc\config.json`
/// Linux/macOS: `~/.config/jellyfin-rpc/config.json`
pub fn get_config_path() -> Result<String, Box<dyn std::error::Error>> {
    debug!("Getting config path");
    if cfg!(not(windows)) {
        debug!("Platform is not Windows");
        let xdg_config_home = match env::var("XDG_CONFIG_HOME") {
            Ok(xdg_config_home) => xdg_config_home,
            Err(_) => env::var("HOME")? + "/.config",
        };

        Ok(xdg_config_home + "/jellyfin-rpc/main.json")
    } else {
        debug!("Platform is Windows");
        let app_data = env::var("APPDATA")?;
        Ok(app_data + r"\jellyfin-rpc\main.json")
    }
}

impl ConfigBuilder {
    fn new() -> Self {
        Self {
            jellyfin: JellyfinBuilder {
                url: "".to_string(),
                username: Username::String("".to_string()),
                api_key: "".to_string(),
                music: None,
                movies: None,
                blacklist: None,
                self_signed_cert: None,
                show_simple: Some(false),
                append_prefix: Some(false),
                add_divider: Some(false),
            },
            discord: None,
            imgur: None,
            images: None,
        }
    }

    /// Loads the config from the given path.
    pub fn load(self, path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        debug!("Config path is: {}", path);

        let data = std::fs::read_to_string(path)?;
        let config = serde_json::from_str(&data)?;

        debug!("Config loaded successfully");

        Ok(config)
    }

    pub fn build(self) -> Config {
        let username = match self.jellyfin.username {
            Username::Vec(usernames) => usernames,
            Username::String(username) => username.split(',').map(|u| u.to_string()).collect(),
        };

        let mut music_state_text = None;
        let mut music_details_text = None;
        let mut music_image_text = None;

        if let Some(music) = self.jellyfin.music {
            music_state_text = music.state_text;
            music_details_text = music.details_text;
            music_image_text = music.image_text;
        }

        let mut movies_state_text = None;
        let mut movies_details_text = None;
        let mut movies_image_text = None;

        if let Some(movies) = self.jellyfin.movies {
            movies_state_text = movies.state_text;
            movies_details_text = movies.details_text;
            movies_image_text = movies.image_text;
        }

        let media_types;
        let libraries;

        if let Some(blacklist) = self.jellyfin.blacklist {
            media_types = blacklist.media_types;
            libraries = blacklist.libraries;
        } else {
            media_types = None;
            libraries = None;
        }

        let application_id;
        let buttons;
        let show_paused;

        if let Some(discord) = self.discord {
            application_id = discord.application_id;
            buttons = discord.buttons;
            show_paused = discord.show_paused.unwrap_or(true)
        } else {
            application_id = None;
            buttons = None;
            show_paused = true;
        }

        let client_id;

        if let Some(imgur) = self.imgur {
            client_id = imgur.client_id;
        } else {
            client_id = None
        }

        let enable_images;
        let imgur_images;

        if let Some(images) = self.images {
            enable_images = images.enable_images.unwrap_or(false);
            imgur_images = images.imgur_images.unwrap_or(false);
        } else {
            enable_images = false;
            imgur_images = false;
        }

        Config {
            jellyfin: Jellyfin {
                url: self.jellyfin.url,
                api_key: self.jellyfin.api_key,
                username,
                music: DisplayOptions {
                    state_text: music_state_text,
                    details_text: music_details_text,
                    image_text: music_image_text,
                },
                movies: DisplayOptions {
                    state_text: movies_state_text,
                    details_text: movies_details_text,
                    image_text: movies_image_text,
                },
                blacklist: Blacklist {
                    media_types,
                    libraries,
                },
                self_signed_cert: self.jellyfin.self_signed_cert.unwrap_or(false),
                show_simple: self.jellyfin.show_simple.unwrap_or(false),
                append_prefix: self.jellyfin.append_prefix.unwrap_or(false),
                add_divider: self.jellyfin.add_divider.unwrap_or(false),
            },
            discord: Discord {
                application_id,
                buttons,
                show_paused,
            },
            imgur: Imgur { client_id },
            images: Images {
                enable_images,
                imgur_images,
            },
        }
    }
}
