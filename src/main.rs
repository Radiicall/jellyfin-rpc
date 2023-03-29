pub mod services;
pub use crate::services::jellyfin::*;
pub use crate::services::imgur::*;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use colored::Colorize;
use clap::Parser;
use retry::retry_with_index;

const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

struct Config {
    url: String,
    api_key: String,
    username: String,
    blacklist: Vec<String>,
    rpc_client_id: String,
    imgur_client_id: String,
    enable_images: bool,
    imgur_images: bool,
}

#[derive(Debug)]
enum ConfigError {
    MissingConfig,
    Io(std::io::Error),
    Var(std::env::VarError),
}

impl From<std::env::VarError> for ConfigError {
    fn from(value: std::env::VarError) -> Self {
        Self::Var(value)
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Parser, Debug)]
#[command(author = "Radical <Radiicall> <radical@radical.fun>")]
#[command(version)]
#[command(about = "Rich presence for Jellyfin", long_about = None)]
struct Args {
    #[arg(short = 'c', long = "config", help = "Path to the config file")]
    config: Option<String>,
    #[arg(short = 'i', long = "image-urls-file", help = "Path to image urls file for imgur")]
    image_urls: Option<String>
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let config_path = args.config.unwrap_or_else(||
        if cfg!(not(windows)) {
            if std::env::var("USER").unwrap() != *"root" {
                std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_|
                    {
                        let mut dir = std::env::var("HOME").unwrap();
                        dir.push_str("/.config/jellyfin-rpc/main.json");
                        dir
                    }
                )
            } else {
                "/etc/jellyfin-rpc/main.json".to_string()
            }
        } else {
            let mut dir = std::env::var("APPDATA").unwrap();
            dir.push_str(r"\jellyfin-rpc\main.json");
            dir
        }
    );

    std::fs::create_dir_all(std::path::Path::new(&config_path).parent().unwrap()).ok();

    if config_path.ends_with(".env") {
        panic!("\n{}\n(Example: https://github.com/Radiicall/jellyfin-rpc/blob/main/example.json)\n", "Please update your .env to JSON format.".bold().red())
    }

    let config = load_config(
        config_path.clone()
    ).unwrap_or_else(|_| panic!("\n\nPlease populate your config file '{}' with the needed variables\n(https://github.com/Radiicall/jellyfin-rpc#setup)\n\n", std::fs::canonicalize(config_path).unwrap().to_string_lossy()));

    println!("{}\n                          {}", "//////////////////////////////////////////////////////////////////".bold(), "Jellyfin-RPC".bright_blue());

    if config.enable_images && !config.imgur_images {
        println!("{}\n{}", "------------------------------------------------------------------".bold(), "Images without Imgur requires port forwarding!".bold().red())
    }
    if config.blacklist[0] != "none" {
        println!("{} {}", "These media types won't be shown:".bold().red(), config.blacklist.join(", ").bold().red())
    }
    let mut blacklist_check: bool = false;
    let mut connected: bool = false;
    let mut rich_presence_client = DiscordIpcClient::new(config.rpc_client_id.as_str()).expect("Failed to create Discord RPC client, discord is down or the Client ID is invalid.");

    // Start up the client connection, so that we can actually send and receive stuff
    connect(&mut rich_presence_client);
    println!("{}\n{}", "Connected to Discord Rich Presence Socket".bright_green().bold(), "------------------------------------------------------------------".bold());

    // Start loop
    loop {
        let mut content = get_jellyfin_playing(&config.url, &config.api_key, &config.username, &config.enable_images).await?;

        config.blacklist.iter().for_each(|x| blacklist_check = !content.media_type.contains(x));

        if !content.media_type.is_empty() && blacklist_check {
            // Print what we're watching
            if !connected {
                println!("\n{}\n{}", content.details.bright_cyan().bold(), content.state_message.bright_cyan().bold());
                // Set connected to true so that we don't try to connect again
                connected = true;
            }
            if config.imgur_images && content.media_type != "livetv" {
                content.image_url = get_image_imgur(&content.image_url, &content.item_id, &config.imgur_client_id, args.image_urls.clone()).await?;
            }
            
            // Set the activity
            let mut rpcbuttons: Vec<activity::Button> = vec![];
            for i in 0..content.external_service_names.len() {
                rpcbuttons.push(activity::Button::new(
                    &content.external_service_names[i],
                    &content.external_service_urls[i],
                ));
            }

            rich_presence_client.set_activity(
                setactivity(&content.state_message, &content.details, content.endtime, &content.image_url, rpcbuttons, format!("Jellyfin-RPC v{}", VERSION.unwrap_or("0.0.0")).as_str(), &content.media_type)
            ).unwrap_or_else(|_| {
                retry_with_index(retry::delay::Exponential::from_millis(1000), |current_try| {
                    println!("{} {}{}", "Attempt".bold().truecolor(225, 69, 0), current_try.to_string().bold().truecolor(225, 69, 0), ": Trying to reconnect".bold().truecolor(225, 69, 0));
                    match rich_presence_client.reconnect() {
                        Ok(result) => retry::OperationResult::Ok(result),
                        Err(_) => {
                            println!("{}", "Failed to reconnect, retrying soon".red().bold());
                            retry::OperationResult::Retry(())
                        },
                    }
                }).unwrap();
                println!("{}\n{}", "Reconnected to Discord Rich Presence Socket".bright_green().bold(), "------------------------------------------------------------------".bold());
                println!("\n{}\n{}", content.details.bright_cyan().bold(), content.state_message.bright_cyan().bold());
            });

        } else if connected {
            // Disconnect from the client
            rich_presence_client.clear_activity().expect("Failed to clear activity");
            // Set connected to false so that we dont try to disconnect again
            connected = false;
            println!("{}\n{}\n{}", "------------------------------------------------------------------".bold(), "Cleared Rich Presence".bright_red().bold(), "------------------------------------------------------------------".bold());
        }

    std::thread::sleep(std::time::Duration::from_millis(750));
    }
}

fn load_config(path: String) -> Result<Config, Box<dyn core::fmt::Debug>> {
    let data = std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("\n\nPlease make the file '{}' and populate it with the needed variables\n(https://github.com/Radiicall/jellyfin-rpc#setup)\n\n", path));
    let res: serde_json::Value = serde_json::from_str(&data).unwrap_or_else(|_| panic!("{}", "\nUnable to parse config file. Is this a json file?\n".red().bold()));

    let jellyfin: serde_json::Value = res["Jellyfin"].clone();
    let discord: serde_json::Value = res["Discord"].clone();
    let imgur: serde_json::Value = res["Imgur"].clone();
    let images: serde_json::Value = res["Images"].clone();

    let url = jellyfin["URL"].as_str().unwrap().to_string();
    let api_key = jellyfin["API_KEY"].as_str().unwrap().to_string();
    let username = jellyfin["USERNAME"].as_str().unwrap().to_string();
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
    let rpc_client_id = discord["APPLICATION_ID"].as_str().unwrap_or("1053747938519679018").to_string();
    let imgur_client_id = imgur["CLIENT_ID"].as_str().unwrap().to_string();
    let enable_images = images["ENABLE_IMAGES"].as_bool().unwrap_or_else(|| 
        panic!(
            "\n{}\n{} {} {} {}\n",
            "ENABLE_IMAGES has to be a bool...".red().bold(),
            "EXAMPLE:".bold(), "true".bright_green().bold(), "not".bold(), "'true'".red().bold()
        )
    );
    let imgur_images = images["IMGUR_IMAGES"].as_bool().unwrap_or_else(|| 
        panic!(
            "\n{}\n{} {} {} {}\n",
            "IMGUR_IMAGES has to be a bool...".red().bold(),
            "EXAMPLE:".bold(), "true".bright_green().bold(), "not".bold(), "'true'".red().bold()
        )
    );

    if rpc_client_id.is_empty() || url.is_empty() || api_key.is_empty() || username.is_empty() {
        return Err(Box::new(ConfigError::MissingConfig))
    }
    Ok(Config {
        url,
        api_key,
        username,
        blacklist,
        rpc_client_id,
        imgur_client_id,
        enable_images,
        imgur_images,
    })
}

fn connect(rich_presence_client: &mut DiscordIpcClient) {
    println!("{}", "------------------------------------------------------------------".bold());
    retry_with_index(retry::delay::Exponential::from_millis(1000), |current_try| {
        println!("{} {}{}", "Attempt".bold().truecolor(225, 69, 0), current_try.to_string().bold().truecolor(225, 69, 0), ": Trying to connect".bold().truecolor(225, 69, 0));
        match rich_presence_client.connect() {
            Ok(result) => retry::OperationResult::Ok(result),
            Err(_) => {
                println!("{}", "Failed to connect, retrying soon".red().bold());
                retry::OperationResult::Retry(())
            },
        }
    }).unwrap();
}

fn setactivity<'a>(state_message: &'a String, details: &'a str, endtime: Option<i64>, img_url: &'a str, rpcbuttons: Vec<activity::Button<'a>>, version: &'a str, media_type: &'a str) -> activity::Activity<'a> {
    let mut new_activity = activity::Activity::new()
        .details(details);

    let mut image_url = "https://s1.qwant.com/thumbr/0x380/0/6/aec9d939d464cc4e3b4c9d7879936fbc61901ccd9847d45c68a3ce2dbd86f0/cover.jpg?u=https%3A%2F%2Farchive.org%2Fdownload%2Fgithub.com-jellyfin-jellyfin_-_2020-09-15_17-17-00%2Fcover.jpg";
    
    if media_type == "livetv" {
        image_url = "https://i.imgur.com/XxdHOqm.png"
    } else if !img_url.is_empty() {
        image_url = img_url;
    }

    let mut assets = activity::Assets::new()
        .large_text(version)
        .large_image(image_url);

    if media_type != "livetv" {
        match endtime {
            Some(time) => {
                new_activity = new_activity.clone().timestamps(activity::Timestamps::new()
                    .end(time)
                );
            },
            None => {
                assets = assets.clone().small_image("https://i.imgur.com/wlHSvYy.png")
                    .small_text("Paused");
            },
        }
    } else if endtime.is_none() {
        assets = assets.clone().small_image("https://i.imgur.com/wlHSvYy.png")
            .small_text("Paused");
    }

    if !state_message.is_empty() {
        new_activity = new_activity.clone().state(state_message);
    }
    if !rpcbuttons.is_empty() {
        new_activity = new_activity.clone().buttons(rpcbuttons);
    }
    new_activity = new_activity.clone().assets(assets);

    new_activity
}
