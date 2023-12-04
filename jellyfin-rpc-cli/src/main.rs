use std::sync::mpsc;
use clap::Parser;
use colored::Colorize;
use discord_rich_presence::DiscordIpcClient;
pub use jellyfin_rpc::prelude::*;
pub use jellyfin_rpc::services::imgur::*;
#[cfg(feature = "updates")]
mod updates;

/*
    TODO: Comments
*/

const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

#[derive(Parser)]
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
    #[arg(
        short = 's',
        long = "suppress-warnings",
        help = "Stops warnings from showing on startup",
        default_value_t = false
    )]
    suppress_warnings: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "updates")]
    updates::checker().await;

    let args = Args::parse();
    let config_path = args.config.unwrap_or_else(|| {
        get_config_path().unwrap_or_else(|err| {
            eprintln!("Error determining config path: {:?}", err);
            std::process::exit(1)
        })
    });

    std::fs::create_dir_all(
        std::path::Path::new(&config_path)
            .parent()
            .expect("Invalid config file path"),
    )
    .ok();

    let config = Config::load(&config_path).unwrap_or_else(|e| {
        eprintln!(
            "{} {}",
            format!(
                "Config can't be loaded: {:?}.\nConfig file should be located at:",
                e
            )
            .red()
            .bold(),
            config_path
        );
        std::process::exit(2)
    });

    println!(
        "{}\n                          {}",
        "//////////////////////////////////////////////////////////////////".bold(),
        "Jellyfin-RPC".bright_blue()
    );

    if !args.suppress_warnings && config.jellyfin.self_signed_cert.is_some_and(|val| val) {
        eprintln!(
            "{}\n{}",
            "------------------------------------------------------------------".bold(),
            "WARNING: Self-signed certificates are enabled!"
                .bold()
                .red()
        );
    }

    if !args.suppress_warnings
        && config
            .clone()
            .images
            .and_then(|images| images.enable_images)
            .unwrap_or(false)
        && !config
            .clone()
            .images
            .and_then(|images| images.imgur_images)
            .unwrap_or(false)
    {
        eprintln!(
            "{}\n{}",
            "------------------------------------------------------------------".bold(),
            "WARNING: Images without Imgur requires port forwarding!"
                .bold()
                .red()
        )
    }
    if config.jellyfin.blacklist.is_some() {
        let blacklist = config.jellyfin.blacklist.clone().unwrap();
        if let Some(media_types) = blacklist.media_types {
            if !media_types.is_empty() {
                println!(
                    "{} {}",
                    "These media types won't be shown:".bold().red(),
                    media_types
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                        .bold()
                        .red()
                )
            }
        }

        if let Some(libraries) = blacklist.libraries {
            if !libraries.is_empty() {
                println!(
                    "{} {}",
                    "These media libraries won't be shown:".bold().red(),
                    libraries.join(", ").bold().red()
                )
            }
        }
    }

    let mut rich_presence_client = DiscordIpcClient::new(
        config
            .discord
            .clone()
            .and_then(|discord| discord.application_id)
            .unwrap_or(String::from("1053747938519679018"))
            .as_str(),
    )
    .expect("Failed to create Discord RPC client, discord is down or the Client ID is invalid.");
    
    let (transmitter, reciever) = mpsc::channel();

    tokio::spawn(async move {
        jellyfin_rpc::presence_loop(transmitter, &mut rich_presence_client, config, VERSION.unwrap_or("0.0.0"), args.image_urls).await.expect("Server crashed");
    });

    loop {
        match reciever.recv() {
            Ok(event) => match event {
                Event::Information(data, color) => match color {
                    Color::Red => println!("{}", data.bold().bright_red()),
                    Color::Green => println!("{}", data.bold().bright_green()),
                    Color::Orange => println!("{}", data.bold().truecolor(225, 69, 0)),
                },
                Event::Activity(details, state_message) => println!("{}\n{}", details.bright_cyan().bold(), state_message.bright_cyan().bold()),
                Event::Spacer => println!("{}", "------------------------------------------------------------------".bold()),
                Event::Error(data, error) => eprintln!("{}\nError: {}", data.bold().bright_red(), error),
            },
            Err(error) => eprintln!("{} {}", "Failed to recieve event".red(), error),
        }
    }
}
