use crate::core::error::ConfigError;
use crate::services::jellyfin::MediaType;
use colored::Colorize;
use std::env;

/*
    TODO: Comments
*/

#[derive(Default)]
struct ConfigBuilder {
    url: String,
    api_key: String,
    username: Vec<String>,
    blacklist: Blacklist,
    music: String,
    rpc_client_id: String,
    imgur_client_id: String,
    images: Images,
}

impl ConfigBuilder {
    fn new() -> Self {
        Self::default()
    }

    fn url(&mut self, url: String) {
        self.url = url;
    }

    fn api_key(&mut self, api_key: String) {
        self.api_key = api_key;
    }

    fn username(&mut self, username: Vec<String>) {
        self.username = username;
    }

    fn blacklist(&mut self, types: Vec<MediaType>, libraries: Vec<String>) {
        self.blacklist = Blacklist { types, libraries };
    }

    fn music(&mut self, music: String) {
        self.music = music
    }

    fn rpc_client_id(&mut self, rpc_client_id: String) {
        self.rpc_client_id = rpc_client_id;
    }

    fn imgur_client_id(&mut self, imgur_client_id: String) {
        self.imgur_client_id = imgur_client_id;
    }

    fn images(&mut self, enabled: bool, imgur: bool) {
        self.images = Images { enabled, imgur };
    }

    fn build(self) -> Result<Config, ConfigError> {
        match (
            self.url.is_empty(),
            self.api_key.is_empty(),
            self.username.is_empty(),
            self.rpc_client_id.is_empty(),
            (self.images.imgur, self.imgur_client_id.is_empty()),
        ) {
            (true, _, _, _, _) => Err(ConfigError::from("Jellyfin URL is empty!")),
            (_, true, _, _, _) => Err(ConfigError::from("Jellyfin API key is empty!")),
            (_, _, true, _, _) => Err(ConfigError::from("Jellyfin Username is empty!")),
            (_, _, _, true, _) => Err(ConfigError::from("Discord Application ID is empty!")),
            (_, _, _, _, (true, true)) => Err(ConfigError::from(
                "Imgur Client ID is empty but Imgur images are enabled!",
            )),
            (false, false, false, false, _) => Ok(Config {
                    url: self.url,
                    api_key: self.api_key,
                    username: self.username,
                    blacklist: self.blacklist,
                    music: self.music,
                    rpc_client_id: self.rpc_client_id,
                    imgur_client_id: self.imgur_client_id,
                    images: self.images
                },
            ),
        }
    }
}

pub struct Config {
    pub url: String,
    pub api_key: String,
    pub username: Vec<String>,
    pub blacklist: Blacklist,
    pub music: String,
    pub rpc_client_id: String,
    pub imgur_client_id: String,
    pub images: Images,
}

#[derive(Default)]
pub struct Blacklist {
    pub types: Vec<MediaType>,
    pub libraries: Vec<String>,
}

#[derive(Default)]
pub struct Images {
    pub enabled: bool,
    pub imgur: bool,
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
        let mut config = ConfigBuilder::new();
        let data = std::fs::read_to_string(path)?;
        let res: serde_json::Value = serde_json::from_str(&data)?;

        let jellyfin: serde_json::Value = res["Jellyfin"].clone();
        let discord: serde_json::Value = res["Discord"].clone();
        let imgur: serde_json::Value = res["Imgur"].clone();
        let images: serde_json::Value = res["Images"].clone();

        config.url(jellyfin["URL"].as_str().unwrap_or("").to_string());
        config.api_key(jellyfin["API_KEY"].as_str().unwrap_or("").to_string());
        if jellyfin["USERNAME"].as_str().is_some() {
            config.username(vec![
                jellyfin["USERNAME"].as_str().unwrap_or("").to_string()
            ]);
        } else {
            let mut usernames: Vec<String> = Vec::new();
            jellyfin["USERNAME"].as_array()
                .unwrap()
                .iter()
                .for_each(|username|
                    usernames.push(username.as_str().unwrap().to_string())
                );
            config.username(usernames)
        }
        let mut type_blacklist: Vec<MediaType> = vec![MediaType::None];
        if jellyfin["TYPE_BLACKLIST"].get(0).is_some() {
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
        if jellyfin["LIBRARY_BLACKLIST"].get(0).is_some() {
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

        config.music(jellyfin["Music"]["Display"].as_str().unwrap_or("genres").to_string());

        config.blacklist(type_blacklist, library_blacklist);
        config.rpc_client_id(discord["APPLICATION_ID"]
            .as_str()
            .unwrap_or("1053747938519679018")
            .to_string());

        config.imgur_client_id(imgur["CLIENT_ID"].as_str().unwrap_or("").to_string());

        config.images(
            images["ENABLE_IMAGES"].as_bool().unwrap_or_else(|| {
                eprintln!(
                    "{}\n{} {} {} {}",
                    "ENABLE_IMAGES has to be a bool...".red().bold(),
                    "EXAMPLE:".bold(),
                    "true".bright_green().bold(),
                    "not".bold(),
                    "'true'".red().bold()
                );
                std::process::exit(2)
            }),
            images["IMGUR_IMAGES"].as_bool().unwrap_or_else(|| {
                eprintln!(
                    "{}\n{} {} {} {}",
                    "IMGUR_IMAGES has to be a bool...".red().bold(),
                    "EXAMPLE:".bold(),
                    "true".bright_green().bold(),
                    "not".bold(),
                    "'true'".red().bold()
                );
                std::process::exit(2)
            })
        );

        config.build()
    }
}
