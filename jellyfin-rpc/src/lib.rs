//! Backend for displaying jellyfin rich presence on discord

/// Main module
pub mod core;
/// Useful imports
///
/// Contains imports that most programs will be using.
pub mod prelude;
/// External connections
pub mod services;
pub use crate::core::error;
pub use core::rpc::{setactivity, presence_loop};
use discord_rich_presence::DiscordIpc;
use discord_rich_presence::DiscordIpcClient;
use retry::retry_with_index;
#[cfg(test)]
mod tests;

/// Function for connecting to the Discord Ipc.
pub fn connect(rich_presence_client: &mut DiscordIpcClient, transmitter: std::sync::mpsc::Sender<core::rpc::Event>) {
    use crate::core::rpc::{Event, Color};
    transmitter.send(Event::Spacer).ok();
    retry_with_index(
        retry::delay::Exponential::from_millis(1000),
        |current_try| {
            transmitter.send(
                Event::Information(format!("Attempt {}: Trying to connect", current_try), Color::Orange)
            ).ok();
            match rich_presence_client.connect() {
                Ok(result) => retry::OperationResult::Ok(result),
                Err(err) => {
                    transmitter.send(Event::Error("Failed to connect, retrying soon".to_string(), err.to_string())).ok();
                    retry::OperationResult::Retry(())
                }
            }
        },
    )
    .unwrap();
}

/// Built in reqwest::get() function, has an extra field to specify if the self signed cert should be accepted.
pub async fn get<U: reqwest::IntoUrl>(
    url: U,
    self_signed_cert: bool,
) -> Result<reqwest::Response, reqwest::Error> {
    reqwest::Client::builder()
        .danger_accept_invalid_certs(self_signed_cert)
        .build()?
        .get(url)
        .send()
        .await
}
