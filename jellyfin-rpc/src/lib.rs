pub mod services;
use discord_rich_presence::DiscordIpc;
use discord_rich_presence::DiscordIpcClient;
use retry::retry_with_index;
pub use crate::services::imgur;
pub use crate::services::jellyfin;
pub mod core;
pub use crate::core::config::{get_config_path, Button, Config};

const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

pub fn connect(rich_presence_client: &mut DiscordIpcClient) {
    retry_with_index(
        retry::delay::Exponential::from_millis(1000),
        |_| {
            match rich_presence_client.connect() {
                Ok(result) => retry::OperationResult::Ok(result),
                Err(_) => {
                    retry::OperationResult::Retry(())
                }
            }
        },
    )
    .unwrap();
}
