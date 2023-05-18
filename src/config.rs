use crate::services::jellyfin::MediaType;
use colored::Colorize;
use std::env;

/*
    TODO: Comments
*/

pub struct Config {
    pub url: String,
    pub api_key: String,
    pub username: String,
    pub blacklist: Blacklist,
    pub rpc_client_id: String,
    pub imgur_client_id: String,
    pub images: Images,
}

pub struct Blacklist {
    pub types: Vec<MediaType>,
    pub libraries: Vec<String>,
}

pub struct Images {
    pub enabled: bool,
    pub imgur: bool,
}

#[derive(Debug)]
pub enum ConfigError {
    MissingConfig(String),
    Io(String),
    Json(String),
    VarError(String),
}

impl From<&'static str> for ConfigError {
    fn from(value: &'static str) -> Self {
        Self::MissingConfig(value.to_string())
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(format!("Unable to open file: {}", value))
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(format!("Unable to parse config: {}", value))
    }
}

impl From<env::VarError> for ConfigError {
    fn from(value: env::VarError) -> Self {
        Self::VarError(format!("Unable to get environment variables: {}", value))
    }
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
        let res: serde_json::Value = serde_json::from_str(&data)?;

        let jellyfin: serde_json::Value = res["Jellyfin"].clone();
        let discord: serde_json::Value = res["Discord"].clone();
        let imgur: serde_json::Value = res["Imgur"].clone();
        let images: serde_json::Value = res["Images"].clone();

        let url = jellyfin["URL"].as_str().unwrap_or("").to_string();
        let api_key = jellyfin["API_KEY"].as_str().unwrap_or("").to_string();
        let username = jellyfin["USERNAME"].as_str().unwrap_or("").to_string();
        let mut type_blacklist: Vec<MediaType> = vec![MediaType::None];
        if !Option::is_none(&jellyfin["TYPE_BLACKLIST"].get(0)) {
            type_blacklist.pop();
            jellyfin["TYPE_BLACKLIST"]
                .as_array()
                .unwrap()
                .iter()
                .for_each(|val| {
                    if val != "music" && val != "movie" && val != "episode" && val != "livetv" {
                        eprintln!("{} is invalid, valid media types to blacklist include: \"music\", \"movie\", \"episode\" and \"livetv\"", val);
                        std::process::exit(2)
                    }
                    type_blacklist.push(
                        MediaType::from(val
                            .as_str()
                            .expect("Media types to blacklist need to be in quotes \"music\"")
                            .to_string()))
                });
        }
        let mut library_blacklist: Vec<String> = vec!["".to_string()];
        if !Option::is_none(&jellyfin["LIBRARY_BLACKLIST"].get(0)) {
            library_blacklist.pop();
            jellyfin["LIBRARY_BLACKLIST"]
                .as_array()
                .unwrap()
                .iter()
                .for_each(|val| {
                    library_blacklist.push(
                        val.as_str()
                            .expect("Libraries to blacklist need to be in quotes \"music\"")
                            .to_lowercase(),
                    )
                });
        }
        let rpc_client_id = discord["APPLICATION_ID"]
            .as_str()
            .unwrap_or("1053747938519679018")
            .to_string();

        let imgur_client_id = imgur["CLIENT_ID"].as_str().unwrap_or("").to_string();

        let enable_images = images["ENABLE_IMAGES"].as_bool().unwrap_or_else(|| {
            eprintln!(
                "{}\n{} {} {} {}",
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
                blacklist: Blacklist {
                    types: type_blacklist,
                    libraries: library_blacklist,
                },
                rpc_client_id,
                imgur_client_id,
                images: Images {
                    enabled: enable_images,
                    imgur: imgur_images,
                },
            }),
        }
    }
}
