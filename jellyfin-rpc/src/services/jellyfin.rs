use crate::core::config::{Config, Display, Username};
use crate::core::error::ContentError;
use async_recursion::async_recursion;
use serde::{de::Visitor, Deserialize, Serialize};
use serde_json::Value;

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

/// Struct containing information about what's being played on jellyfin
#[derive(Default)]
pub struct Content {
    /// What type of content is currently playing.
    ///
    /// Example: MediaType::Movie
    pub media_type: MediaType,
    /// The title of the content
    pub details: String,
    /// Description of the content, usually includes season and episode/genres, etc.
    pub state_message: String,
    /// When the content will end, current UNIX epoch + time left
    pub endtime: Option<i64>,
    /// Image URL supplied by Jellyfin, this is different from the Imgur URL
    ///
    /// This has to be passed to the Imgur::get() function to upload images to imgur
    pub image_url: String,
    /// Item ID of the content currently playing,
    /// used to store Imgur URLs so that they can be reused instead of reuploading to Imgur every time.
    pub item_id: String,
    /// External services to display as buttons.
    ///
    /// Example: IMDb, Trakt, etc.
    pub external_services: Vec<ExternalServices>,
}

impl Content {
    /// Calls the Content::get() function recursively until it returns a Content struct.
    ///
    /// It waits (attempt * 5) seconds before retrying.
    ///
    /// The max time it will wait is 30 seconds.
    #[async_recursion]
    pub async fn try_get(config: &Config, attempt: u64) -> Self {
        let mut time = attempt * 5;
        if time > 30 {
            time = 30
        }

        match Content::get(config).await {
            Ok(content) => content,
            Err(error) => {
                eprintln! {"{}: Error while getting content: {:#?}", attempt, error}
                while time > 0 {
                    eprint!("\rRetrying in {} seconds…", time);
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    time -= 1
                }
                eprintln!();
                Content::try_get(config, attempt + 1).await
            }
        }
    }

    /// Returns a Content struct with the updated information from jellyfin
    pub async fn get(config: &Config) -> Result<Self, ContentError> {
        let sessions: Vec<Value> = serde_json::from_str(
            &crate::get(
                format!(
                    "{}/Sessions?api_key={}",
                    config.jellyfin.url.trim_end_matches('/'),
                    config.jellyfin.api_key
                ),
                config.jellyfin.self_signed_cert.unwrap_or(false),
            )
            .await?
            .text()
            .await?,
        )?;
        for session in sessions {
            if session.get("UserName").is_none() {
                continue;
            }

            let session_username = session["UserName"].as_str().unwrap().to_lowercase();

            match &config.jellyfin.username {
                Username::String(username) if session_username != username.to_lowercase() => {
                    continue
                }
                Username::Vec(usernames)
                    if usernames
                        .iter()
                        .all(|username| session_username != username.to_lowercase()) =>
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
            let play_state = &session["PlayState"];

            Content::watching(&mut content, now_playing_item, play_state, config).await;

            // Check that details and state_message arent over the max length allowed by discord, if they are then they have to be trimmed down because discord wont display the activity otherwise
            if content.details.len() > 128 {
                content.details(content.details.chars().take(128).collect());
            }

            if content.state_message.len() > 128 {
                content.state_message(content.state_message.chars().take(128).collect());
            }

            if content.details.len() < 3 {
                let current_details = content.details.clone();

                content.details(current_details + "‎‎");
            }

            let mut image_url: String = "".to_string();
            if config
                .images
                .as_ref()
                .and_then(|images| images.enable_images)
                .unwrap_or(false)
            {
                image_url = Content::image(
                    &config.jellyfin.url,
                    content.item_id.clone(),
                    config.jellyfin.self_signed_cert.unwrap_or(false),
                )
                .await
                .unwrap_or(String::from(""));
            }

            content.external_services(ExternalServices::get(now_playing_item).await);

            if content.media_type != MediaType::Book {
                content.endtime(Content::time_left(now_playing_item, play_state).await);
            }
            content.image_url(image_url);

            return Ok(content.build());
        }
        Ok(Self::default())
    }

    async fn watching(
        content: &mut ContentBuilder,
        now_playing_item: &Value,
        play_state: &Value,
        config: &Config,
    ) -> Option<()> {
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
        let name = if config.jellyfin.show_simple? {
          ""
        } else {
          now_playing_item["Name"].as_str()?
        };
        if now_playing_item["Type"].as_str()? == "Episode" {
            let season = now_playing_item["ParentIndexNumber"].to_string();
            let first_episode_number = now_playing_item["IndexNumber"].to_string();
            let mut state = "S".to_owned() + &season + "E" + &first_episode_number;

            if season == *"null" {
                state = "E".to_owned() + &first_episode_number;
            }

            if now_playing_item.get("IndexNumberEnd").is_some() {
                state += &("-".to_string() + &now_playing_item["IndexNumberEnd"].to_string());
            }

            if !config.jellyfin.show_simple? {
              state += &(" ".to_string() + name);
            }
            content.media_type(MediaType::Episode);
            content.details(now_playing_item["SeriesName"].as_str()?.to_string());
            content.state_message(state);
            content.item_id(now_playing_item["SeriesId"].as_str()?.to_string());
        } else if now_playing_item["Type"].as_str()? == "Movie" {
            let genres = Content::get_genres(now_playing_item).unwrap_or(String::from(""));

            content.media_type(MediaType::Movie);
            content.details(name.into());
            content.state_message(genres);
            content.item_id(now_playing_item["Id"].as_str()?.to_string());
        } else if now_playing_item["Type"].as_str()? == "Audio" {
            if let Some(extratype) = now_playing_item.get("ExtraType").and_then(Value::as_str) {
                if extratype == "ThemeSong" {
                    return Some(());
                }
            }
            let raw_artists = now_playing_item["Artists"]
                .as_array()?
                .iter()
                .map(|a| a.as_str().unwrap())
                .collect::<Vec<&str>>();

            let artists = Self::get_artists(raw_artists);

            let display = match config
                .jellyfin
                .music
                .clone()
                .and_then(|music| music.display)
            {
                Some(Display::Vec(music)) => music,
                Some(Display::String(music)) => music
                    .split(',')
                    .map(|d| d.trim().to_string())
                    .collect::<Vec<String>>(),
                _ => vec![String::from("genres")],
            };

            let separator = config
                .jellyfin
                .music
                .clone()
                .and_then(|music| music.separator)
                .unwrap_or("-".to_string());

            let state =
                Content::get_music_info(now_playing_item, artists, display, name, &separator).await;

            content.media_type(MediaType::Music);
            content.details(name.into());
            content.state_message(state);
            content.item_id(
                now_playing_item["AlbumId"]
                    .as_str()
                    .unwrap_or(now_playing_item["Id"].as_str()?)
                    .to_string(),
            );
        } else if now_playing_item["Type"].as_str()? == "TvChannel" {
            content.media_type(MediaType::LiveTv);
            content.details(name.into());
            content.state_message("Live TV".into());
            content.item_id(now_playing_item["Id"].as_str()?.to_string());
        } else if now_playing_item["Type"].as_str()? == "Book" {
            // Time to convert jellyfin nonsense into pages!!!
            let ticks_to_pages = 10000;

            let mut position_ticks = play_state["PositionTicks"].as_i64().unwrap_or(0);
            position_ticks /= ticks_to_pages;

            content.state_message(format!("Reading page {}", position_ticks));
            content.media_type(MediaType::Book);
            content.details(name.into());
            content.item_id(now_playing_item["Id"].as_str()?.to_string());
        } else if now_playing_item["Type"].as_str()? == "AudioBook" {
            let raw_artists = now_playing_item["Artists"]
                .as_array()?
                .iter()
                .map(|a| a.as_str().unwrap())
                .collect::<Vec<&str>>();

            let artists = Self::get_artists(raw_artists);

            let mut genres = Content::get_genres(now_playing_item).unwrap_or(String::from(""));

            if !genres.is_empty() {
                genres = String::from(" - ") + &genres
            }

            content.media_type(MediaType::AudioBook);
            content.item_id(now_playing_item["ParentId"].as_str()?.to_string());
            content.details(now_playing_item["Album"].as_str().unwrap_or(name).into());
            content.state_message(format!("By {}{}", artists, genres))
        }
        Some(())
    }

    fn get_artists(raw_artists: Vec<&str>) -> String {
        let mut artists = String::new();
        for (i, artist) in raw_artists.iter().enumerate() {
            if i != 0 {
                if i == raw_artists.len() - 1 {
                    artists += " and ";
                } else {
                    artists += ", "
                }
            }
            artists += artist
        }
        artists
    }

    async fn time_left(now_playing_item: &Value, play_state: &Value) -> Option<i64> {
        if !play_state["IsPaused"].as_bool()? {
            let ticks_to_seconds = 10000000;

            let mut position_ticks = play_state["PositionTicks"].as_i64().unwrap_or(0);
            position_ticks /= ticks_to_seconds;

            let mut runtime_ticks = now_playing_item["RunTimeTicks"].as_i64().unwrap_or(0);
            runtime_ticks /= ticks_to_seconds;

            Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .ok()?
                    .as_secs() as i64
                    + (runtime_ticks - position_ticks),
            )
        } else {
            None
        }
    }

    async fn get_music_info(
        npi: &Value,
        artists: String,
        display: Vec<std::string::String>,
        name: &str,
        separator: &str,
    ) -> String {
        let mut state = format!("By {}", artists);

        display.iter().for_each(|data| {
            let data = data.as_str();

            match data {
                "genres" => {
                    if let Some(genres) = Content::get_genres(npi) {
                        state.push_str(&format!(" {} ", separator));
                        state.push_str(&genres)
                    }
                }

                "album" if npi["Album"].as_str().unwrap_or("") != name => {
                    state.push_str(&format!(" {} ", separator));
                    state.push_str(npi["Album"].as_str().unwrap_or(""));
                }

                "year" if npi["ProductionYear"].as_u64().unwrap_or(0) != 0 => {
                    state.push_str(&format!(" {} ", separator));
                    state.push_str(&npi["ProductionYear"].as_u64().unwrap().to_string());
                }

                _ => (),
            }
        });
        state
    }

    async fn image(
        url: &str,
        item_id: String,
        self_signed_cert: bool,
    ) -> Result<String, reqwest::Error> {
        let img = format!(
            "{}/Items/{}/Images/Primary",
            url.trim_end_matches('/'),
            item_id
        );

        if crate::get(&img, self_signed_cert)
            .await?
            .text()
            .await
            .unwrap_or(String::from("_"))
            .contains("does not have an image of type Primary")
        {
            Ok(String::from(""))
        } else {
            Ok(img)
        }
    }

    fn get_genres(npi: &Value) -> Option<String> {
        match npi.get("Genres") {
            Some(genre_array) if !genre_array.as_array()?.is_empty() => {
                let mut genres = String::new();
                genres.push_str(
                    &genre_array
                        .as_array()?
                        .iter()
                        .map(|genre| genre.as_str().unwrap())
                        .collect::<Vec<&str>>()
                        .join(", "),
                );
                Some(genres)
            }
            Some(_) => None,
            None => None,
        }
    }
}

/// Struct with the external services collected from Jellyfin.
#[derive(Debug, Clone)]
pub struct ExternalServices {
    /// Name of the service
    ///
    /// Example: IMDb, Trakt
    pub name: String,
    /// URL pointing to the specific Show/Movie etc. on the external service.
    ///
    /// Example: <https://www.imdb.com/title/tt0117500/>, <https://trakt.tv/shows/the-simpsons>
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

/// The type of the currently playing content.
#[derive(Debug, PartialEq, Clone)]
pub enum MediaType {
    /// If the content playing is a Movie.
    Movie,
    /// If the content playing is an Episode.
    Episode,
    /// If the content playing is a LiveTv.
    LiveTv,
    /// If the content playing is a Music.
    Music,
    /// If the content playing is a Book.
    Book,
    /// If the content playing is an Audio Book.
    AudioBook,
    /// If nothing is playing.
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
            MediaType::Book => serializer.serialize_unit_variant("MediaType", 4, "Book"),
            MediaType::AudioBook => serializer.serialize_unit_variant("MediaType", 4, "AudioBook"),
            MediaType::None => serializer.serialize_unit_variant("MediaType", 5, "None"),
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
        Ok(MediaType::from(v.to_lowercase()))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(MediaType::from(v.to_lowercase()))
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(MediaType::from(v.to_lowercase()))
    }
}

impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            MediaType::Episode => "Episode",
            MediaType::LiveTv => "LiveTv",
            MediaType::Movie => "Movie",
            MediaType::Music => "Music",
            MediaType::Book => "Book",
            MediaType::AudioBook => "AudioBook",
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
    /// Check if the MediaType is none, returns `true` if it is.
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
            "book" => Self::Book,
            "audiobook" => Self::AudioBook,
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
            "book" => Self::Book,
            "audiobook" => Self::AudioBook,
            _ => Self::None,
        }
    }
}

/// Checks if a library exists.
// TODO: Reverse the output of this function, currently returns `false` if the library exists.
pub async fn library_check(
    url: &str,
    api_key: &str,
    item_id: &str,
    library: &str,
    self_signed_cert: bool,
) -> Result<bool, Box<dyn std::error::Error>> {
    let parents: Vec<Value> = serde_json::from_str(
        &crate::get(
            format!(
                "{}/Items/{}/Ancestors?api_key={}",
                url.trim_end_matches('/'),
                item_id,
                api_key
            ),
            self_signed_cert,
        )
        .await?
        .text()
        .await?,
    )?;

    for i in parents {
        if let Some(name) = i.get("Name").and_then(Value::as_str) {
            if name.to_lowercase() == library.to_lowercase() {
                return Ok(false);
            }
        }
    }

    Ok(true)
}
