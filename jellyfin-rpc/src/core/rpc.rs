use crate::prelude::*;
use discord_rich_presence::{activity, DiscordIpcClient, DiscordIpc};
use retry::retry_with_index;
use std::sync::mpsc;
#[cfg(feature = "imgur")]
use crate::services::imgur::*;

/// Used to set the activity on Discord.
///
/// This has checks to do different things for different mediatypes and replaces images with default ones if they are needed.
pub fn setactivity<'a>(
    state_message: &'a str,
    details: &'a str,
    endtime: Option<i64>,
    img_url: &'a str,
    rpcbuttons: Vec<activity::Button<'a>>,
    version: &'a str,
    media_type: &'a MediaType,
) -> activity::Activity<'a> {
    let mut new_activity = activity::Activity::new().details(details);

    let mut image_url = "https://i.imgur.com/oX6vcds.png";

    if media_type == &MediaType::LiveTv {
        image_url = "https://i.imgur.com/XxdHOqm.png"
    } else if !img_url.is_empty() {
        image_url = img_url;
    }

    let mut assets = activity::Assets::new()
        .large_text(version)
        .large_image(image_url);

    match endtime {
        Some(_) if media_type == &MediaType::LiveTv => (),
        Some(time) => {
            new_activity = new_activity
                .clone()
                .timestamps(activity::Timestamps::new().end(time));
        }
        None if media_type == &MediaType::Book => (),
        None => {
            assets = assets
                .clone()
                .small_image("https://i.imgur.com/wlHSvYy.png")
                .small_text("Paused");
        }
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

pub async fn presence_loop<'a>(transmitter: mpsc::Sender<Event>,rich_presence_client: &mut DiscordIpcClient, config: Config, version: &'a str, image_urls: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let mut connected = false;

    // Start up the client connection, so that we can actually send and receive stuff
    crate::connect(rich_presence_client, transmitter.clone());

    transmitter.send(Event::Information("Connected to Discord Rich Presence Socket".to_string(), Color::Green)).ok();

    // Start loop
    loop {
        let mut content = Content::try_get(&config, 1).await;

        let mut blacklist_check = true;
        config
            .clone()
            .jellyfin
            .blacklist
            .and_then(|blacklist| blacklist.media_types)
            .unwrap_or(vec![MediaType::None])
            .iter()
            .for_each(|x| {
                if blacklist_check && !content.media_type.is_none() {
                    blacklist_check = content.media_type != *x
                }
            });
        if config
            .clone()
            .jellyfin
            .blacklist
            .and_then(|blacklist| blacklist.libraries)
            .is_some()
        {
            for library in &config
                .clone()
                .jellyfin
                .blacklist
                .and_then(|blacklist| blacklist.libraries)
                .unwrap()
            {
                if blacklist_check && !content.media_type.is_none() {
                    blacklist_check = jellyfin::library_check(
                        &config.jellyfin.url,
                        &config.jellyfin.api_key,
                        &content.item_id,
                        library,
                        config.jellyfin.self_signed_cert.unwrap_or(false),
                    )
                    .await?;
                }
            }
        }

        if !content.media_type.is_none() && blacklist_check {
            // Print what we're watching
            if !connected {
                transmitter.send(Event::Activity(content.details.clone(), content.state_message.clone())).ok();
                // Set connected to true so that we don't try to connect again
                connected = true;
            }
            #[cfg(feature = "imgur")]
            if config
                .clone()
                .images
                .and_then(|images| images.imgur_images)
                .unwrap_or(false)
                && content.media_type != MediaType::LiveTv
            {
                content.image_url = Imgur::get(
                    &content.image_url,
                    &content.item_id,
                    &config
                        .clone()
                        .imgur
                        .and_then(|imgur| imgur.client_id)
                        .expect("Imgur client ID cant be loaded."),
                    image_urls.clone(),
                    config.jellyfin.self_signed_cert.unwrap_or(false),
                )
                .await
                .unwrap_or_else(|e| {
                    transmitter.send(Event::Error("Failed to use Imgur".to_string(), format!("{:?}", e))).ok();
                    Imgur::default()
                })
                .url;
            }

            // Set the activity
            let mut rpcbuttons: Vec<activity::Button> = vec![];
            let mut x = 0;
            let default_button = config::Button {
                name: String::from("dynamic"),
                url: String::from("dynamic"),
            };
            let buttons = config
                .clone()
                .discord
                .and_then(|discord| discord.buttons)
                .unwrap_or(vec![default_button.clone(), default_button]);

            // For loop to determine if external services are to be used or if there are custom buttons instead
            for button in buttons.iter() {
                if button.name == "dynamic"
                    && button.url == "dynamic"
                    && content.external_services.len() != x
                {
                    rpcbuttons.push(activity::Button::new(
                        &content.external_services[x].name,
                        &content.external_services[x].url,
                    ));
                    x += 1
                } else if button.name != "dynamic" || button.url != "dynamic" {
                    rpcbuttons.push(activity::Button::new(&button.name, &button.url))
                }
            }

            rich_presence_client
                .set_activity(crate::setactivity(
                    &content.state_message,
                    &content.details,
                    content.endtime,
                    &content.image_url,
                    rpcbuttons,
                    format!("Jellyfin-RPC v{}", version).as_str(),
                    &content.media_type,
                ))
                .unwrap_or_else(|err| {
                    transmitter.send(Event::Error("Failed to set activity".to_string(), err.to_string())).ok();
                    retry_with_index(
                        retry::delay::Exponential::from_millis(1000),
                        |current_try| {
                            transmitter.send(
                                Event::Information(format!("Attempt {}: Trying to reconnect", current_try), Color::Orange)
                            ).ok();

                            match rich_presence_client.reconnect() {
                                Ok(result) => retry::OperationResult::Ok(result),
                                Err(err) => {
                                    transmitter.send(Event::Error("Failed to reconnect, retrying soon".to_string(), err.to_string())).ok();
                                    retry::OperationResult::Retry(())
                                }
                            }
                        },
                    )
                    .unwrap();
                    transmitter.send(Event::Information("Reconnected to Discord Rich Presence Socket".to_string(), Color::Green)).ok();
                    transmitter.send(Event::Spacer).ok();
                    transmitter.send(Event::Activity(content.details, content.state_message)).ok();
                });
        } else if connected {
            // Disconnect from the client
            rich_presence_client
                .clear_activity()
                .expect("Failed to clear activity");
            // Set connected to false so that we dont try to disconnect again
            connected = false;

            transmitter.send(Event::Spacer).ok();
            transmitter.send(Event::Information("Cleared Rich Presence".to_string(), Color::Red)).ok();
            transmitter.send(Event::Spacer).ok();
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }
}

pub enum Event {
    Information(String, Color),
    Activity(String, String),
    Spacer,
    Error(String, String),
}

pub enum Color {
    Red,
    Green,
    Orange,
}
