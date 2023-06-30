use crate::core::error::ConfigError;
use crate::services::jellyfin::MediaType;
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
    music: Music,
    button: Vec<Button>,
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

    fn music_display(&mut self, display: Vec<String>) {
        self.music.display = display
    }

    fn music_seperator(&mut self, separator: Option<char>) {
        self.music.separator = separator
    }

    fn button(&mut self, name: String, url: String) {
        self.button.push(Button { name, url })
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
                music: Music {
                    display: self.music.display,
                    separator: self.music.separator,
                },
                button: self.button,
                rpc_client_id: self.rpc_client_id,
                imgur_client_id: self.imgur_client_id,
                images: self.images,
            }),
        }
    }
}

pub struct Config {
    pub url: String,
    pub api_key: String,
    pub username: Vec<String>,
    pub blacklist: Blacklist,
    pub music: Music,
    pub button: Vec<Button>,
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

#[derive(Default)]
pub struct Music {
    pub display: Vec<String>,
    pub separator: Option<char>,
}

#[derive(Default, Clone)]
pub struct Button {
    pub name: String,
    pub url: String,
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

        let jellyfin: serde_json::Value = res["jellyfin"].clone();
        let music: serde_json::Value = jellyfin["music"].clone();
        let blacklist: serde_json::Value = jellyfin["blacklist"].clone();

        let discord: serde_json::Value = res["discord"].clone();
        let buttons: serde_json::Value = discord["buttons"].clone();

        let imgur: serde_json::Value = res["imgur"].clone();
        let images: serde_json::Value = res["images"].clone();

        config.url(jellyfin["url"].as_str().unwrap_or("").to_string());
        config.api_key(jellyfin["api_key"].as_str().unwrap_or("").to_string());
        if jellyfin["username"].is_string() {
            config.username(vec![jellyfin["username"]
                .as_str()
                .unwrap_or("")
                .to_string()]);
        } else {
            config.username(
                jellyfin["username"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|username| username.as_str().unwrap().to_string())
                    .collect::<Vec<String>>(),
            );
        }
        let mut library_blacklist: Vec<String> = vec!["".to_string()];
        let mut type_blacklist: Vec<MediaType> = vec![MediaType::None];
        if blacklist.is_object() {
            if blacklist["media_types"].get(0).is_some() {
                type_blacklist.pop();
                blacklist["media_types"]
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

            if blacklist["libraries"].get(0).is_some() {
                library_blacklist.pop();
                blacklist["libraries"]
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
        }
        config.blacklist(type_blacklist, library_blacklist);

        if music["display"].is_string() {
            config.music_display(
                music["display"]
                    .as_str()
                    .unwrap()
                    .split(',')
                    .map(|username| username.trim().to_string())
                    .collect::<Vec<String>>(),
            )
        } else if music["display"].is_array() {
            config.music_display(
                music["display"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|username| username.as_str().unwrap().trim().to_string())
                    .collect::<Vec<String>>(),
            )
        } else {
            config.music_display(vec![String::from("genres")])
        }

        config.music_seperator(music["separator"].as_str().unwrap_or("-").chars().next());

        if buttons.is_array() {
            let buttons = buttons.as_array().unwrap();
            for button in buttons {
                if let (Some(name), Some(url)) = (
                    button.get("name").and_then(serde_json::Value::as_str),
                    button.get("url").and_then(serde_json::Value::as_str),
                ) {
                    config.button(name.into(), url.into());
                }
                if config.button.len() == 2 {
                    break;
                }
            }
        } else {
            config.button("dynamic".into(), "dynamic".into());
            config.button("dynamic".into(), "dynamic".into());
        }

        config.rpc_client_id(
            discord["application_id"]
                .as_str()
                .unwrap_or("1053747938519679018")
                .to_string(),
        );

        config.imgur_client_id(imgur["client_id"].as_str().unwrap_or("").to_string());

        config.images(
            images["enable_images"].as_bool().unwrap_or(false),
            images["imgur_images"].as_bool().unwrap_or(false),
        );

        config.build()
    }
}
