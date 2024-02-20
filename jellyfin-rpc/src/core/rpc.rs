use super::config::Discord;
use crate::prelude::MediaType;
use discord_rich_presence::activity;

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

pub fn show_paused<'a>(
    media_type: &'a MediaType,
    endtime: Option<i64>,
    discord: &'a Option<Discord>,
) -> bool {
    if media_type == &MediaType::Book {
        return true;
    }

    if endtime.is_some() {
        return true;
    }

    if let Some(discord) = discord {
        return discord.show_paused.unwrap_or(true);
    }

    true
}
