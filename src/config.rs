use colored::Colorize;
use std::env;

/*
    TODO: Comments
*/

pub struct Config {
    pub url: String,
    pub api_key: String,
    pub username: String,
    pub blacklist: Vec<String>,
    pub rpc_client_id: String,
    pub imgur_client_id: String,
    pub enable_images: bool,
    pub imgur_images: bool,
}

#[derive(Debug)]
pub enum ConfigError {
    MissingConfig(String),
    Io(String),
    Json(String),
}

impl From<&'static str> for ConfigError {
    fn from(value: &'static str) -> Self {
        Self::MissingConfig(value.to_string())
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(format!("Unable to parse config: {}", value))
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(format!("Unable to read file: {}", value))
    }
}

pub fn get_config_path() -> Result<String, String> {
    if cfg!(not(windows)) {
        let user = env::var("USER").map_err(|e| e.to_string())?;
        if user != "root" {
            let xdg_config_home = env::var("XDG_CONFIG_HOME")
                .unwrap_or_else(|_| env::var("HOME").unwrap() + "/.config");
            Ok(xdg_config_home + ("/jellyfin-rpc/main.json"))
        } else {
            Ok("/etc/jellyfin-rpc/main.json".to_string())
        }
    } else {
        let app_data = env::var("APPDATA").map_err(|e| e.to_string())?;
        Ok(app_data + r"\jellyfin-rpc\main.json")
    }
}

impl Config {
    pub fn load_config(path: String) -> Result<Config, ConfigError> {
        let data = std::fs::read_to_string(path)?;
        let res: serde_json::Value = serde_json::from_str(&data)?;

        let jellyfin: serde_json::Value = res["Jellyfin"].clone();
        let discord: serde_json::Value = res["Discord"].clone();
        let imgur: serde_json::Value = res["Imgur"].clone();
        let images: serde_json::Value = res["Images"].clone();

        let url = jellyfin["URL"].as_str().unwrap_or("").to_string();
        let api_key = jellyfin["API_KEY"].as_str().unwrap_or("").to_string();
        let username = jellyfin["USERNAME"].as_str().unwrap_or("").to_string();
        let mut blacklist: Vec<String> = vec!["none".to_string()];
        if !Option::is_none(&jellyfin["BLACKLIST"].get(0)) {
            blacklist.pop();
            jellyfin["BLACKLIST"]
                .as_array()
                .unwrap()
                .iter()
                .for_each(|val| {
                    if val != "music" && val != "movie" && val != "episode" && val != "livetv" {
                        panic!("Valid media types to blacklist include: 'music', 'movie', 'episode' and 'livetv'")
                    }
                    blacklist.push(
                        val
                            .as_str()
                            .expect("Media types to blacklist need to be in quotes \"music\"")
                            .to_string())
                });
        }
        let rpc_client_id = discord["APPLICATION_ID"]
            .as_str()
            .unwrap_or("1053747938519679018")
            .to_string();
        let imgur_client_id = imgur["CLIENT_ID"].as_str().unwrap_or("").to_string();
        let enable_images = images["ENABLE_IMAGES"].as_bool().unwrap_or_else(|| {
            eprintln!(
                "\n{}\n{} {} {} {}\n",
                "ENABLE_IMAGES has to be a bool...".red().bold(),
                "EXAMPLE:".bold(),
                "true".bright_green().bold(),
                "not".bold(),
                "'true'".red().bold()
            );
            std::process::exit(2)
        });
        let imgur_images = images["IMGUR_IMAGES"].as_bool().unwrap_or_else(|| {
            eprintln!(
                "{}\n{} {} {} {}",
                "IMGUR_IMAGES has to be a bool...".red().bold(),
                "EXAMPLE:".bold(),
                "true".bright_green().bold(),
                "not".bold(),
                "'true'".red().bold()
            );
            std::process::exit(2)
        });

        match (
            url.is_empty(),
            api_key.is_empty(),
            username.is_empty(),
            rpc_client_id.is_empty(),
            (imgur_images, imgur_client_id.is_empty()),
        ) {
            (true, _, _, _, _) => Err(ConfigError::from("Jellyfin URL is empty!")),
            (_, true, _, _, _) => Err(ConfigError::from("Jellyfin API key is empty!")),
            (_, _, true, _, _) => Err(ConfigError::from("Jellyfin Username is empty!")),
            (_, _, _, true, _) => Err(ConfigError::from("Discord Application ID is empty!")),
            (_, _, _, _, (true, true)) => Err(ConfigError::from(
                "Imgur Client ID is empty but Imgur images are enabled!",
            )),
            (false, false, false, false, _) => Ok(Config {
                url,
                api_key,
                username,
                blacklist,
                rpc_client_id,
                imgur_client_id,
                enable_images,
                imgur_images,
            }),
        }
    }
}
