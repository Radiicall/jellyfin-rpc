use crate::core::config::{Config, Display, Username};
use serde::{de::Visitor, Deserialize, Serialize};
use serde_json::Value;

/*
    TODO: Comments
*/

#[derive(Default, Clone)]
struct ContentBuilder {
    media_type: MediaType,
    details: String,
    state_message: String,
    endtime: Option<i64>,
    image_url: String,
    item_id: String,
    external_services: Vec<ExternalServices>,
}

impl ContentBuilder {
    fn new() -> Self {
        Self::default()
    }

    fn media_type(&mut self, media_type: MediaType) {
        self.media_type = media_type;
    }

    fn details(&mut self, details: String) {
        self.details = details;
    }

    fn state_message(&mut self, state_message: String) {
        self.state_message = state_message;
    }

    fn endtime(&mut self, endtime: Option<i64>) {
        self.endtime = endtime;
    }

    fn image_url(&mut self, image_url: String) {
        self.image_url = image_url;
    }

    fn item_id(&mut self, item_id: String) {
        self.item_id = item_id;
    }

    fn external_services(&mut self, external_services: Vec<ExternalServices>) {
        self.external_services = external_services;
    }

    pub fn build(self) -> Content {
        Content {
            media_type: self.media_type,
            details: self.details,
            state_message: self.state_message,
            endtime: self.endtime,
            image_url: self.image_url,
            item_id: self.item_id,
            external_services: self.external_services,
        }
    }
}

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
    pub async fn get(config: &Config) -> Result<Self, reqwest::Error> {
        let sessions: Vec<Value> = serde_json::from_str(
            &reqwest::get(format!(
                "{}/Sessions?api_key={}",
                config.jellyfin.url.trim_end_matches('/'),
                config.jellyfin.api_key
            ))
            .await?
            .text()
            .await?,
        )
        .unwrap_or_else(|_| {
            panic!(
                "Can't unwrap URL, check if JELLYFIN_URL is correct. Current URL: {}",
                config.jellyfin.url
            )
        });
        for session in sessions {
            if session.get("UserName").is_none() {
                continue;
            }

            match &config.jellyfin.username {
                Username::String(username) if session["UserName"].as_str().unwrap() != username => {
                    continue
                }
                Username::Vec(usernames)
                    if usernames
                        .iter()
                        .all(|username| session["UserName"].as_str().unwrap() != username) =>
                {
                    continue
                }
                _ => (),
            };

            if session.get("NowPlayingItem").is_none() {
                continue;
            }

            let mut content = ContentBuilder::new();

            let now_playing_item = &session["NowPlayingItem"];

            Content::watching(&mut content, now_playing_item, config).await;

            let mut image_url: String = "".to_string();
            if config
                .images
                .as_ref()
                .and_then(|images| images.enable_images)
                .unwrap_or(false)
            {
                image_url = Content::image(&config.jellyfin.url, content.item_id.clone()).await;
            }
            content.external_services(ExternalServices::get(now_playing_item).await);
            content.endtime(Content::time_left(now_playing_item, &session).await);
            content.image_url(image_url);

            return Ok(content.build());
        }
        Ok(Self::default())
    }

    async fn watching(content: &mut ContentBuilder, now_playing_item: &Value, config: &Config) {
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
        let mut genres = "".to_string();
        if now_playing_item["Type"].as_str().unwrap() == "Episode" {
            let season = now_playing_item["ParentIndexNumber"].to_string();
            let first_episode_number = now_playing_item["IndexNumber"].to_string();
            let mut state = "S".to_owned() + &season + "E" + &first_episode_number;

            if now_playing_item.get("IndexNumberEnd").is_some() {
                state += &("-".to_string() + &now_playing_item["IndexNumberEnd"].to_string());
            }

            state += &(" ".to_string() + name);
            content.media_type(MediaType::Episode);
            content.details(now_playing_item["SeriesName"].as_str().unwrap().to_string());
            content.state_message(state);
            content.item_id(now_playing_item["SeriesId"].as_str().unwrap().to_string());
        } else if now_playing_item["Type"].as_str().unwrap() == "Movie" {
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

            content.media_type(MediaType::Movie);
            content.details(name.into());
            content.state_message(genres);
            content.item_id(now_playing_item["Id"].as_str().unwrap().to_string());
        } else if now_playing_item["Type"].as_str().unwrap() == "Audio" {
            let artist = now_playing_item["AlbumArtist"]
                .as_str()
                .unwrap()
                .to_string();

            let display = match config
                .jellyfin
                .music
                .clone()
                .and_then(|music| music.display)
            {
                Some(Display::Vec(music)) => music,
                Some(Display::String(music)) => music
                    .split(",")
                    .map(|d| d.trim().to_string())
                    .collect::<Vec<String>>(),
                _ => vec![String::from("genres")],
            };

            let separator = config
                .jellyfin
                .music
                .clone()
                .and_then(|music| music.separator)
                .unwrap_or('-');

            let mut state = format!("By {} - ", artist);
            let mut index = 0;
            display.iter().for_each(|data| {
                index += 1;
                let data = data.as_str();
                let old_state = state.clone();
                match data {
                    "genres" => match now_playing_item.get("Genres") {
                        None => (),
                        genre_array => state.push_str(
                            &genre_array
                                .unwrap()
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|genre| genre.as_str().unwrap().to_string())
                                .collect::<Vec<String>>()
                                .join(", "),
                        ),
                    },
                    "album" => state.push_str(now_playing_item["Album"].as_str().unwrap_or("")),
                    "year" => {
                        let mut year = now_playing_item["ProductionYear"]
                            .as_u64()
                            .unwrap_or(0)
                            .to_string();
                        if year == "0" {
                            year = String::from("");
                        }
                        state.push_str(&year)
                    }
                    _ => state = format!("By {}", artist),
                }

                if state != old_state && display.len() != index {
                    state.push_str(&format!(" {} ", separator))
                }
            });

            content.media_type(MediaType::Music);
            content.details(name.into());
            content.state_message(state);
            content.item_id(now_playing_item["AlbumId"].as_str().unwrap().to_string());
        } else if now_playing_item["Type"].as_str().unwrap() == "TvChannel" {
            content.media_type(MediaType::LiveTv);
            content.details(name.into());
            content.state_message("Live TV".into());
            content.item_id(now_playing_item["Id"].as_str().unwrap().to_string());
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

#[derive(Debug, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
pub enum MediaType {
    Movie,
    Episode,
    LiveTv,
    Music,
    None,
}

impl Serialize for MediaType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match *self {
            MediaType::Movie => serializer.serialize_unit_variant("MediaType", 0, "Movie"),
            MediaType::Episode => serializer.serialize_unit_variant("MediaType", 1, "Episode"),
            MediaType::LiveTv => serializer.serialize_unit_variant("MediaType", 2, "LiveTv"),
            MediaType::Music => serializer.serialize_unit_variant("MediaType", 3, "Music"),
            MediaType::None => serializer.serialize_unit_variant("MediaType", 4, "None"),
        }
    }
}

impl<'de> Deserialize<'de> for MediaType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_string(MediaTypeVisitor)
    }
}

struct MediaTypeVisitor;

impl<'de> Visitor<'de> for MediaTypeVisitor {
    type Value = MediaType;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(MediaType::from(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match v.to_lowercase().as_str() {
            "movie" => Ok(MediaType::Movie),
            "episode" => Ok(MediaType::Episode),
            "livetv" => Ok(MediaType::LiveTv),
            "music" => Ok(MediaType::Music),
            _ => Ok(MediaType::None),
        }
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match v.to_lowercase().as_str() {
            "movie" => Ok(MediaType::Movie),
            "episode" => Ok(MediaType::Episode),
            "livetv" => Ok(MediaType::LiveTv),
            "music" => Ok(MediaType::Music),
            _ => Ok(MediaType::None),
        }
    }
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
        self == &Self::None
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
            if name.to_lowercase() == library.to_lowercase() {
                return false;
            }
        }
    }

    true
}
