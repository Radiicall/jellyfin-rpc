pub mod services;
pub use crate::services::imgur::*;
pub use crate::services::jellyfin::*;
pub mod config;
pub use crate::config::*;
use clap::Parser;
use colored::Colorize;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use retry::retry_with_index;

/*
    TODO: Comments
*/

const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug)]
#[command(author = "Radical <Radiicall> <radical@radical.fun>")]
#[command(version)]
#[command(about = "Rich presence for Jellyfin", long_about = None)]
struct Args {
    #[arg(short = 'c', long = "config", help = "Path to the config file")]
    config: Option<String>,
    #[arg(
        short = 'i',
        long = "image-urls-file",
        help = "Path to image urls file for imgur"
    )]
    image_urls: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let config_path = args.config.unwrap_or_else(|| {
        get_config_path().unwrap_or_else(|err| {
            eprintln!("Error determining config path: {}", err);
            std::process::exit(1)
        })
    });

    std::fs::create_dir_all(std::path::Path::new(&config_path).parent().unwrap()).ok();

    let config = Config::load_config(config_path.clone()).unwrap_or_else(|e| {
        eprintln!(
            "{}",
            format!("Config can't be loaded: {:?}", e).red().bold()
        );
        std::process::exit(2)
    });

    println!(
        "{}\n                          {}",
        "//////////////////////////////////////////////////////////////////".bold(),
        "Jellyfin-RPC".bright_blue()
    );

    if config.enable_images && !config.imgur_images {
        eprintln!(
            "{}\n{}",
            "------------------------------------------------------------------".bold(),
            "Images without Imgur requires port forwarding!"
                .bold()
                .red()
        )
    }
    if config.blacklist[0] != "none" {
        println!(
            "{} {}",
            "These media types won't be shown:".bold().red(),
            config.blacklist.join(", ").bold().red()
        )
    }
    let mut blacklist_check: bool = false;
    let mut connected: bool = false;
    let mut rich_presence_client = DiscordIpcClient::new(config.rpc_client_id.as_str()).expect(
        "Failed to create Discord RPC client, discord is down or the Client ID is invalid.",
    );

    // Start up the client connection, so that we can actually send and receive stuff
    connect(&mut rich_presence_client);
    println!(
        "{}\n{}",
        "Connected to Discord Rich Presence Socket"
            .bright_green()
            .bold(),
        "------------------------------------------------------------------".bold()
    );

    // Start loop
    loop {
        let mut content = get_jellyfin_playing(
            &config.url,
            &config.api_key,
            &config.username,
            &config.enable_images,
        )
        .await?;

        config
            .blacklist
            .iter()
            .for_each(|x| blacklist_check = !content.media_type.contains(x));

        if !content.media_type.is_empty() && blacklist_check {
            // Print what we're watching
            if !connected {
                println!(
                    "\n{}\n{}",
                    content.details.bright_cyan().bold(),
                    content.state_message.bright_cyan().bold()
                );
                // Set connected to true so that we don't try to connect again
                connected = true;
            }
            if config.imgur_images && content.media_type != "livetv" {
                content.image_url = get_image_imgur(
                    &content.image_url,
                    &content.item_id,
                    &config.imgur_client_id,
                    args.image_urls.clone(),
                )
                .await?;
            }

            // Set the activity
            let mut rpcbuttons: Vec<activity::Button> = vec![];
            for i in 0..content.external_service_names.len() {
                rpcbuttons.push(activity::Button::new(
                    &content.external_service_names[i],
                    &content.external_service_urls[i],
                ));
            }

            rich_presence_client
                .set_activity(setactivity(
                    &content.state_message,
                    &content.details,
                    content.endtime,
                    &content.image_url,
                    rpcbuttons,
                    format!("Jellyfin-RPC v{}", VERSION.unwrap_or("0.0.0")).as_str(),
                    &content.media_type,
                ))
                .unwrap_or_else(|_| {
                    retry_with_index(
                        retry::delay::Exponential::from_millis(1000),
                        |current_try| {
                            println!(
                                "{} {}{}",
                                "Attempt".bold().truecolor(225, 69, 0),
                                current_try.to_string().bold().truecolor(225, 69, 0),
                                ": Trying to reconnect".bold().truecolor(225, 69, 0)
                            );
                            match rich_presence_client.reconnect() {
                                Ok(result) => retry::OperationResult::Ok(result),
                                Err(_) => {
                                    eprintln!(
                                        "{}",
                                        "Failed to reconnect, retrying soon".red().bold()
                                    );
                                    retry::OperationResult::Retry(())
                                }
                            }
                        },
                    )
                    .unwrap();
                    println!(
                        "{}\n{}",
                        "Reconnected to Discord Rich Presence Socket"
                            .bright_green()
                            .bold(),
                        "------------------------------------------------------------------".bold()
                    );
                    println!(
                        "\n{}\n{}",
                        content.details.bright_cyan().bold(),
                        content.state_message.bright_cyan().bold()
                    );
                });
        } else if connected {
            // Disconnect from the client
            rich_presence_client
                .clear_activity()
                .expect("Failed to clear activity");
            // Set connected to false so that we dont try to disconnect again
            connected = false;
            println!(
                "{}\n{}\n{}",
                "------------------------------------------------------------------".bold(),
                "Cleared Rich Presence".bright_red().bold(),
                "------------------------------------------------------------------".bold()
            );
        }

        std::thread::sleep(std::time::Duration::from_millis(750));
    }
}

fn connect(rich_presence_client: &mut DiscordIpcClient) {
    println!(
        "{}",
        "------------------------------------------------------------------".bold()
    );
    retry_with_index(
        retry::delay::Exponential::from_millis(1000),
        |current_try| {
            println!(
                "{} {}{}",
                "Attempt".bold().truecolor(225, 69, 0),
                current_try.to_string().bold().truecolor(225, 69, 0),
                ": Trying to connect".bold().truecolor(225, 69, 0)
            );
            match rich_presence_client.connect() {
                Ok(result) => retry::OperationResult::Ok(result),
                Err(_) => {
                    eprintln!("{}", "Failed to connect, retrying soon".red().bold());
                    retry::OperationResult::Retry(())
                }
            }
        },
    )
    .unwrap();
}

fn setactivity<'a>(
    state_message: &'a String,
    details: &'a str,
    endtime: Option<i64>,
    img_url: &'a str,
    rpcbuttons: Vec<activity::Button<'a>>,
    version: &'a str,
    media_type: &'a str,
) -> activity::Activity<'a> {
    let mut new_activity = activity::Activity::new().details(details);

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
                new_activity = new_activity
                    .clone()
                    .timestamps(activity::Timestamps::new().end(time));
            }
            None => {
                assets = assets
                    .clone()
                    .small_image("https://i.imgur.com/wlHSvYy.png")
                    .small_text("Paused");
            }
        }
    } else if endtime.is_none() {
        assets = assets
            .clone()
            .small_image("https://i.imgur.com/wlHSvYy.png")
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
