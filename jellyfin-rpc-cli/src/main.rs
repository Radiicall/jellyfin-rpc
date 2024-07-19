use std::time::Duration;
use clap::Parser;
use colored::Colorize;
use config::{get_config_path, Config, Username};
use jellyfin_rpc::Client;
use log::{error, info, warn};
use retry::retry_with_index;
use simple_logger::SimpleLogger;
use time::macros::format_description;
use tokio::time::sleep;
#[cfg(feature = "updates")]
mod updates;
mod config;

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
        short = 't',
        long = "wait-time",
        help = "Time to wait between loops in seconds",
        default_value_t = 3
    )]
    wait_time: usize,
    #[arg(
        short = 'v',
        long = "log-level",
        help = "Sets the log level to one of: trace, debug, info, warn, error, off",
        default_value_t = String::from("info")
    )]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if std::env::var("RUST_LOG").is_err() {
        let _ = tokio::task::spawn_blocking(move || {
            std::env::set_var("RUST_LOG", args.log_level);
        })
        .await;
    }

    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .env()
        .with_timestamp_format(format_description!(
            "[year]-[month]-[day] [hour]:[minute]:[second]"
        ))
        .init()
        .unwrap();

    info!("Initializing Jellyfin-RPC");

    #[cfg(feature = "updates")]
    updates::checker().await;

    let conf = Config::builder()
        .load(&args.config.unwrap_or(get_config_path()?))?
        .build();

    let mut builder = Client::builder();
    builder
        .api_key(conf.jellyfin.api_key)
        .url(conf.jellyfin.url)
        .usernames(conf.jellyfin.username)
        .self_signed(conf.jellyfin.self_signed_cert)
        .episode_simple(conf.jellyfin.show_simple)
        .episode_divider(conf.jellyfin.add_divider)
        .episode_prefix(conf.jellyfin.append_prefix)
        .show_paused(conf.discord.show_paused)
        .show_images(conf.images.enable_images)
        .use_imgur(conf.images.imgur_images);

    if let Some(display) = conf.jellyfin.music.display {
        builder.music_display(display);
    }

    if let Some(separator) = conf.jellyfin.music.separator {
        builder.music_separator(separator);
    }

    if let Some(media_types) = conf.jellyfin.blacklist.media_types {
        builder.blacklist_media_types(media_types);
    }

    if let Some(libraries) = conf.jellyfin.blacklist.libraries {
        builder.blacklist_libraries(libraries);
    }

    if let Some(application_id) = conf.discord.application_id {
        builder.client_id(application_id);
    }

    if let Some(buttons) = conf.discord.buttons {
        builder.buttons(buttons);
    }

    if let Some(client_id) = conf.imgur.client_id {
        builder.imgur_client_id(client_id);
    }
    
    let mut client = builder.build()?;

    client.connect().await?;

    loop {
        if let Err(err) = client.set_activity().await {
            error!("{}", err);
            warn!("Retrying...");
            client.reconnect().await?;
            client.set_activity().await?;
        }
        sleep(Duration::from_secs(args.wait_time as u64)).await;
    }
}
