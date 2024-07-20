use std::{
    thread::sleep,
    time::Duration
};
use clap::Parser;
use config::{get_config_path, get_urls_path, Config};
use jellyfin_rpc::Client;
use log::{debug, error, info};
use retry::retry_with_index;
use simple_logger::SimpleLogger;
use time::macros::format_description;
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if std::env::var("RUST_LOG").is_err() {
        let _ = std::env::set_var("RUST_LOG", args.log_level);
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
    updates::checker();

    let conf = Config::builder()
        .load(&args.config.unwrap_or(get_config_path()?))?
        .build();

    debug!("Creating jellyfin-rpc client builder");
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
        .use_imgur(conf.images.imgur_images)
        .large_image_text(format!("Jellyfin-RPC v{}", VERSION.unwrap_or("UNKNOWN")))
        .imgur_urls_file_location(args.image_urls.unwrap_or(get_urls_path()?));

    if let Some(display) = conf.jellyfin.music.display {
        debug!("Found config.jellyfin.music.display");
        builder.music_display(display);
    }

    if let Some(separator) = conf.jellyfin.music.separator {
        debug!("Found config.jellyfin.music.separator");
        builder.music_separator(separator);
    }

    if let Some(display) = conf.jellyfin.movies.display {
        debug!("Found config.jellyfin.music.display");
        builder.movies_display(display);
    }

    if let Some(separator) = conf.jellyfin.movies.separator {
        debug!("Found config.jellyfin.music.separator");
        builder.movies_separator(separator);
    }

    if let Some(media_types) = conf.jellyfin.blacklist.media_types {
        debug!("Found config.jellyfin.blacklist.media_types");
        builder.blacklist_media_types(media_types);
    }

    if let Some(libraries) = conf.jellyfin.blacklist.libraries {
        debug!("Found config.jellyfin.blacklist.libraries");
        builder.blacklist_libraries(libraries);
    }

    if let Some(application_id) = conf.discord.application_id {
        debug!("Found config.discord.application_id");
        builder.client_id(application_id);
    }

    if let Some(buttons) = conf.discord.buttons {
        debug!("Found config.discord.buttons");
        builder.buttons(buttons);
    }

    if let Some(client_id) = conf.imgur.client_id {
        debug!("Found config.imgur.client_id");
        builder.imgur_client_id(client_id);
    }
    
    debug!("Building client");
    let mut client = builder.build()?;

    info!("Connecting to Discord");
    retry_with_index(retry::delay::Exponential::from_millis(1000), |current_try| {
        info!("Attempt {}: Trying to connect", current_try);
        match client.connect() {
            Ok(_) => retry::OperationResult::Ok(()),
            Err(err) => {
                error!("{}", err);
                retry::OperationResult::Retry(())
            },
        }
    }).unwrap();
    info!("Connected!");

    let mut currently_playing = String::new();

    loop {
        match client.set_activity() {
            Ok(activity) => {
                if activity.is_empty() && !currently_playing.is_empty() {
                    let _ = client.clear_activity();
                    info!("Cleared activity");
                    currently_playing = activity;
                } else if activity != currently_playing {
                    currently_playing = activity;

                    info!("{}", currently_playing);
                }

            },
            Err(err) => {
                error!("{}", err);
                retry_with_index(retry::delay::Exponential::from_millis(1000), |current_try| {
                    info!("Attempt {}: Trying to reconnect", current_try);
                    match client.reconnect() {
                        Ok(_) => retry::OperationResult::Ok(()),
                        Err(err) => {
                            error!("{}", err);
                            retry::OperationResult::Retry(())
                        },
                    }
                }).unwrap();
                info!("Reconnected!");
            
                client.set_activity()?;
            },
        }

        sleep(Duration::from_secs(args.wait_time as u64));
    }
}
