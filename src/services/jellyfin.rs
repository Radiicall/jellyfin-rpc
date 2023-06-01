use serde_json::Value;

/*
    TODO: Comments
*/

#[derive(Default)]
pub struct Content {
    pub media_type: MediaType,
    pub details: String,
    pub state_message: String,
    pub endtime: Option<i64>,
    pub image_url: String,
    pub item_id: String,
    pub external_services: Vec<ExternalServices>,
}

impl Content {
    pub async fn get(
        url: &str,
        api_key: &str,
        username: &str,
        enable_images: &bool,
    ) -> Result<Self, reqwest::Error> {
        let sessions: Vec<Value> = serde_json::from_str(
            &reqwest::get(format!(
                "{}/Sessions?api_key={}",
                url.trim_end_matches('/'),
                api_key
            ))
            .await?
            .text()
            .await?,
        )
        .unwrap_or_else(|_| {
            panic!(
                "Can't unwrap URL, check if JELLYFIN_URL is correct. Current URL: {}",
                url
            )
        });
        for session in sessions {
            if Option::is_none(&session.get("UserName")) {
                continue;
            }
            if session["UserName"].as_str().unwrap() != username {
                continue;
            }
            if Option::is_none(&session.get("NowPlayingItem")) {
                continue;
            }

            let now_playing_item = &session["NowPlayingItem"];

            let mut content = Content::watching(now_playing_item).await;

            content.external_services = ExternalServices::get(now_playing_item).await;

            content.endtime = Content::time_left(now_playing_item, &session).await;

            let mut image_url: String = "".to_string();
            if enable_images == &true {
                image_url = Content::image(url, content.item_id.clone()).await;
            }
            content.image_url = image_url;

            return Ok(content);
        }
        Ok(Self::default())
    }

    async fn watching(now_playing_item: &Value) -> Self {
        /*
        FIXME: Update this explanation/remove it.

        This is where we actually get the info for the Movie/Series that we're currently watching.
        First we set the name variable because that's not gonna change either way.
        Then we check if its an "Episode" or a "Movie".
        If its an "Episode" then we set the item type to "episode", get the name of the series, the season and the actual episode number.
        Then we send that off as a Vec<String> along with the external urls and end timer to the main loop.
        If its a "Movie" then we try to fetch the "Genres" with a simple for loop!
        After the for loop is complete we remove the trailing ", " because it looks bad in the presence.
        Then we send it off as a Vec<String> with the external urls and the end timer to the main loop.
        */
        let name = now_playing_item["Name"].as_str().unwrap();
        let media_type: MediaType;
        let item_id: String;
        let mut genres = "".to_string();
        if now_playing_item["Type"].as_str().unwrap() == "Episode" {
            media_type = MediaType::Episode;
            let series_name = now_playing_item["SeriesName"].as_str().unwrap().to_string();
            item_id = now_playing_item["SeriesId"].as_str().unwrap().to_string();

            let season = now_playing_item["ParentIndexNumber"].to_string();
            let first_episode_number = now_playing_item["IndexNumber"].to_string();
            let mut msg = "S".to_owned() + &season + "E" + &first_episode_number;

            if !Option::is_none(&now_playing_item.get("IndexNumberEnd")) {
                msg += &("-".to_string() + &now_playing_item["IndexNumberEnd"].to_string());
            }

            msg += &(" ".to_string() + name);

            Self {
                media_type, 
                details: series_name, 
                state_message: msg, 
                item_id,
                ..Default::default()
            }
        } else if now_playing_item["Type"].as_str().unwrap() == "Movie" {
            media_type = MediaType::Movie;
            item_id = now_playing_item["Id"].as_str().unwrap().to_string();
            match now_playing_item.get("Genres") {
                None => (),
                genre_array => {
                    genres = genre_array
                        .unwrap()
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|x| x.as_str().unwrap().to_string())
                        .collect::<Vec<String>>()
                        .join(", ");
                }
            };

            Self {
                media_type, 
                details: name.into(), 
                state_message: genres, 
                item_id,
                ..Default::default()
            }
        } else if now_playing_item["Type"].as_str().unwrap() == "Audio" {
            media_type = MediaType::Music;
            item_id = now_playing_item["AlbumId"].as_str().unwrap().to_string();
            let artist = now_playing_item["AlbumArtist"].as_str().unwrap();
            let msg = match now_playing_item.get("Genres") {
                None => format!("By {}", artist),
                genre_array => {
                    format!(
                        "{} - {}",
                        artist,
                        &genre_array
                            .unwrap()
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|x| x.as_str().unwrap().to_string())
                            .collect::<Vec<String>>()
                            .join(", ")
                    )
                }
            };

            Self {
                media_type,
                details: name.into(),
                state_message: msg,
                item_id,
                ..Default::default()
            }
        } else if now_playing_item["Type"].as_str().unwrap() == "TvChannel" {
            media_type = MediaType::LiveTv;
            item_id = now_playing_item["Id"].as_str().unwrap().to_string();
            let msg = "Live TV".to_string();

            Self {
                media_type,
                details: name.into(),
                state_message: msg,
                item_id,
                ..Default::default()
            }
        } else {
            // Return empty struct
            Self::default()
        }
    }

    async fn time_left(now_playing_item: &Value, session: &Value) -> Option<i64> {
        if !session["PlayState"]["IsPaused"].as_bool().unwrap() {
            let ticks_to_seconds = 10000000;

            let mut position_ticks = session["PlayState"]["PositionTicks"].as_i64().unwrap_or(0);
            position_ticks /= ticks_to_seconds;

            let mut runtime_ticks = now_playing_item["RunTimeTicks"].as_i64().unwrap_or(0);
            runtime_ticks /= ticks_to_seconds;

            Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64
                    + (runtime_ticks - position_ticks),
            )
        } else {
            None
        }
    }

    async fn image(url: &str, item_id: String) -> String {
        format!(
            "{}/Items/{}/Images/Primary",
            url.trim_end_matches('/'),
            item_id
        )
    }
}

#[derive(Debug)]
pub struct ExternalServices {
    pub name: String,
    pub url: String,
}

impl ExternalServices {
    async fn get(now_playing_item: &Value) -> Vec<Self> {
        let mut external_services: Vec<Self> = vec![];

        let _external_services = &now_playing_item["ExternalUrls"];

        if let Some(external_urls) = now_playing_item
            .get("ExternalUrls")
            .and_then(Value::as_array)
        {
            for i in external_urls {
                if let (Some(name), Some(url)) = (
                    i.get("Name").and_then(Value::as_str),
                    i.get("Url").and_then(Value::as_str),
                ) {
                    external_services.push(Self {
                        name: name.into(),
                        url: url.into(),
                    });
                    if external_services.len() == 2 {
                        break;
                    }
                }
            }
        }
        external_services
    }
}

#[derive(PartialEq, Clone)]
pub enum MediaType {
    Movie,
    Episode,
    LiveTv,
    Music,
    None,
}

impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            MediaType::Episode => "Episode",
            MediaType::LiveTv => "LiveTv",
            MediaType::Movie => "Movie",
            MediaType::Music => "Music",
            MediaType::None => "None",
        };
        write!(f, "{}", res)
    }
}

impl Default for MediaType {
    fn default() -> Self {
        Self::None
    }
}

impl MediaType {
    pub fn is_none(&self) -> bool {
        if self == &Self::None {
            return true;
        }
        false
    }

    pub fn equal_to(&self, value: String) -> bool {
        self == &Self::from(value)
    }
}

impl From<&'static str> for MediaType {
    fn from(value: &'static str) -> Self {
        match value {
            "episode" => Self::Episode,
            "movie" => Self::Movie,
            "music" => Self::Music,
            "livetv" => Self::LiveTv,
            _ => Self::None,
        }
    }
}

impl From<String> for MediaType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "episode" => Self::Episode,
            "movie" => Self::Movie,
            "music" => Self::Music,
            "livetv" => Self::LiveTv,
            _ => Self::None,
        }
    }
}

pub async fn library_check(url: &str, api_key: &str, item_id: &str, library: &str) -> bool {
    let parents: Vec<Value> = serde_json::from_str(
        &reqwest::get(format!(
            "{}/Items/{}/Ancestors?api_key={}",
            url.trim_end_matches('/'),
            item_id,
            api_key
        ))
        .await
        .unwrap()
        .text()
        .await
        .unwrap(),
    )
    .unwrap_or_else(|_| {
        panic!(
            "Can't unwrap URL, check if JELLYFIN_URL is correct. Current URL: {}",
            url
        )
    });

    for i in parents {
        if let Some(name) = i.get("Name").and_then(Value::as_str) {
            if name.to_lowercase() == library {
                return false;
            }
        }
    }

    true
}
