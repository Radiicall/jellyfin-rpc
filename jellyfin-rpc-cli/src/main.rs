use clap::Parser;
use colored::Colorize;
use config::{get_config_path, Config, Username};
use jellyfin_rpc::Client;
use log::{error, info, warn};
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
        short = 's',
        long = "suppress-warnings",
        help = "Stops warnings from showing on startup",
        default_value_t = false
    )]
    suppress_warnings: bool,
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

    let conf = Config::load(&args.config.unwrap_or(get_config_path()?))?;

    let mut builder = Client::builder();
    builder
        .api_key(conf.jellyfin.api_key)
        .url(conf.jellyfin.url);

    match conf.jellyfin.username {
        Username::Vec(usernames) => builder.usernames(usernames),
        Username::String(username) => builder.username(username),
    };

    if let Some(music) = conf.jellyfin.music {
        if let Some(display) = music.display {
            // I think config needs to be a builder
        }

    }
    
    Ok(())
}
