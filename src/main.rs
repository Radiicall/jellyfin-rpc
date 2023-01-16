pub mod jellyfin;
pub use crate::jellyfin::*;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use colored::Colorize;
use clap::Parser;
use retry::retry_with_index;

struct Config {
    rpc_client_id: String,
    url: String,
    api_key: String,
    username: String,
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
#[command(author = "Radiicall <radical@radical.fun>")]
#[command(version)]
#[command(about = "Rich presence for Jellyfin", long_about = None)]
struct Args {
    #[arg(short = 'c', long = "config", help = "Path to the config file")]
    config: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    dotenv::from_path(
        args.config.unwrap_or_else(|| 
            std::env::current_exe().unwrap()
            .parent().unwrap()
            .join(".env").to_string_lossy().to_string()
        )
    ).ok();
    let config = load_config().expect("Please make a file called .env and populate it with the needed variables (https://github.com/Radiicall/jellyfin-rpc#setup)");

    println!("{}\n                          {}", "//////////////////////////////////////////////////////////////////".bold(), "Jellyfin-RPC".bright_blue());

    let mut connected: bool = false;
    let mut rich_presence_client = DiscordIpcClient::new(config.rpc_client_id.as_str()).expect("Failed to create Discord RPC client, discord is down or the Client ID is invalid.");
    // Start loop
    loop {
        let content = get_jellyfin_playing(&config.url, &config.api_key, &config.username).await.unwrap();

        if !content.media_type.is_empty() {
            if !connected {
                // Start up the client connection, so that we can actually send and receive stuff
                connect(&mut rich_presence_client);
                println!("{}\n{}\n{}\n{}", "Connected to Discord RPC client".bright_green().bold(), "------------------------------------------------------------------".bold(), content.details.bright_cyan().bold(), content.state_message.bright_cyan().bold());

                // Set connected to true so that we don't try to connect again
                connected = true;
            }

            // Set the activity
            let mut rpcbuttons: Vec<activity::Button> = std::vec::Vec::new();
            for i in 0..content.external_service_names.len() {
                rpcbuttons.push(activity::Button::new(
                    &content.external_service_names[i],
                    &content.external_service_urls[i],
                ));
            }
            
            rich_presence_client.set_activity(
                setactivity(&content.state_message, &content.details, content.endtime, rpcbuttons)
            ).expect("Failed to set activity");
            
        } else if connected {
            // Disconnect from the client
            rich_presence_client.close().expect("Failed to close Discord RPC client");
            // Set connected to false so that we dont try to disconnect again
            connected = false;
            println!("{}", "Disconnected from Discord RPC client".bright_red().bold());
        }
    // Sleep for 2 seconds
    std::thread::sleep(std::time::Duration::from_millis(750));
    }
}

fn load_config() -> Result<Config, Box<dyn core::fmt::Debug>> {
    let rpc_client_id = dotenv::var("DISCORD_APPLICATION_ID").unwrap_or_else(|_| "1053747938519679018".to_string());
    let url = dotenv::var("JELLYFIN_URL").unwrap_or_else(|_| "".to_string());
    let api_key = dotenv::var("JELLYFIN_API_KEY").unwrap_or_else(|_| "".to_string());
    let username = dotenv::var("JELLYFIN_USERNAME").unwrap_or_else(|_| "".to_string());
    
    if rpc_client_id.is_empty() || url.is_empty() || api_key.is_empty() || username.is_empty() {
        return Err(Box::new(ConfigError::MissingConfig))
    }
    Ok(Config {
        rpc_client_id,
        url,
        api_key,
        username,
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

fn setactivity<'a>(state_message: &'a String, details: &'a str, endtime: i64, rpcbuttons: Vec<activity::Button<'a>>) -> activity::Activity<'a> {
    let mut new_activity = activity::Activity::new()
        .details(details)
        .assets(
            activity::Assets::new()
                .large_image("https://s1.qwant.com/thumbr/0x380/0/6/aec9d939d464cc4e3b4c9d7879936fbc61901ccd9847d45c68a3ce2dbd86f0/cover.jpg?u=https%3A%2F%2Farchive.org%2Fdownload%2Fgithub.com-jellyfin-jellyfin_-_2020-09-15_17-17-00%2Fcover.jpg")
                .large_text("https://github.com/Radiicall/jellyfin-rpc")
        )
        .timestamps(activity::Timestamps::new()
            .end(endtime)
        );

    if !state_message.is_empty() {
        new_activity = new_activity.clone().state(state_message);
    }
    if !rpcbuttons.is_empty() {
        new_activity = new_activity.clone().buttons(rpcbuttons);
    }
    new_activity
}
