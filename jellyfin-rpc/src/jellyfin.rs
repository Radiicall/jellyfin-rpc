use serde::{de::Visitor, Deserialize, Serialize};
use std::time::{SystemTime, SystemTimeError, UNIX_EPOCH};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct RawSession {
    pub user_name: String,
    pub now_playing_item: Option<NowPlayingItem>,
    pub play_state: PlayState,
}

impl RawSession {
    pub fn build(self) -> Session {
        //TODO: Figure out how to avoid this clone
        let now_playing_item = self.now_playing_item.clone().unwrap();
        let id;

        match now_playing_item.media_type {
            MediaType::Episode => {
                id = now_playing_item.series_id
                    .unwrap_or(now_playing_item.id)
            },
            MediaType::Music => {
                id = now_playing_item.album_id
                    .unwrap_or(now_playing_item.id)
            },
            _ => {
                id = now_playing_item.id
            }
        };

        Session {
            now_playing_item: self.now_playing_item.unwrap(),
            play_state: self.play_state,
            item_id: id.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Session {
    pub now_playing_item: NowPlayingItem,
    pub play_state: PlayState,
    pub item_id: String,
}

impl Session {
    pub fn get_details(&self) -> &str {
        match self.now_playing_item.media_type {
            MediaType::Episode => self.now_playing_item.series_name.as_ref().unwrap_or(&self.now_playing_item.name),
            MediaType::AudioBook => self.now_playing_item.album.as_ref().unwrap_or(&self.now_playing_item.name),
            _ => &self.now_playing_item.name,
        }
    }

    /// Formats artists with comma separation and a final "and" before the last name.
    pub fn format_artists(&self) -> String {
        // let default is to create a longer lived value for artists_vec
        let default = vec!["".to_string()];
        let artists_vec = self.now_playing_item.artists.as_ref().unwrap_or(&default);
        let mut artists = String::new();

        for i in 0..artists_vec.len() {
            if i == 0 {
                artists += &artists_vec[i];
                continue
            }

            if i == artists_vec.len() - 1 {
                artists += &format!(" and {}", artists_vec[i]);
                continue
            }

            artists += &format!(", {}", artists_vec[i]);
        }

        artists
    }

    pub fn get_endtime(&self) -> Result<EndTime, SystemTimeError> {
        match self.now_playing_item.media_type {
            MediaType::Book => return Ok(EndTime::NoEndTime),
            MediaType::LiveTv => return Ok(EndTime::NoEndTime),
            _ => {}
        }

        if !self.play_state.is_paused {
            let ticks_to_seconds = 10000000;

            if let Some(mut position_ticks) = self.play_state.position_ticks {
                position_ticks /= ticks_to_seconds;

                let runtime_ticks = self.now_playing_item.run_time_ticks / ticks_to_seconds;

                return Ok(
                    EndTime::Some(SystemTime::now()
                        .duration_since(UNIX_EPOCH)?
                        .as_secs() as i64
                        + (runtime_ticks - position_ticks))
                )
            }
        }
        Ok(EndTime::Paused)
    }
}

#[derive(PartialEq)]
pub enum EndTime {
    Some(i64),
    NoEndTime,
    Paused,
}

/// Button struct
///
/// Contains information about buttons
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Button {
    /// What the name should be showed as in Discord.
    pub name: String,
    /// What clicking it should point to in Discord.
    pub url: String,
}

impl Default for Button {
    fn default() -> Self {
        Self {
            name: String::from("dynamic"),
            url: String::from("dynamic"),
        }
    }
}

impl Button {
    pub fn new(name: String, url: String) -> Self {
        Self {
            name,
            url,
        }
    }

    pub fn is_dynamic(&self) -> bool {
        self.name == "dynamic" && self.url == "dynamic"
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct NowPlayingItem {
    // Generic
    pub name: String,
    #[serde(rename = "Type")]
    pub media_type: MediaType,
    pub id: String,
    pub run_time_ticks: i64,
    pub production_year: Option<i64>,
    pub genres: Option<Vec<String>>,
    pub external_urls: Option<Vec<ExternalUrl>>,
    pub critic_rating: Option<i64>,
    pub community_rating: Option<f64>,
    // Episode related
    pub parent_index_number: Option<i32>,
    pub index_number: Option<i32>,
    pub index_number_end: Option<i32>,
    pub series_name: Option<String>,
    pub series_id: Option<String>,
    // Audio related
    pub artists: Option<Vec<String>>,
    pub extra_type: Option<String>,
    pub album_id: Option<String>,
    pub album: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct ExternalUrl {
    pub name: String,
    pub url: String,
}

/// The type of the currently playing content.
#[derive(Debug, PartialEq, Clone, Copy)]
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

impl From<&'static str> for MediaType {
    fn from(value: &'static str) -> Self {
        match value {
            "episode" => Self::Episode,
            "movie" => Self::Movie,
            "music" | "audio" => Self::Music,
            "livetv" | "tvchannel" => Self::LiveTv,
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
            "music" | "audio" => Self::Music,
            "livetv" | "tvchannel" => Self::LiveTv,
            "book" => Self::Book,
            "audiobook" => Self::AudioBook,
            _ => Self::None,
        }
    }
}


#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct PlayState {
    pub is_paused: bool,
    pub position_ticks: Option<i64>,

}

#[derive(Deserialize, Debug)]
pub struct Item {
    pub name: Option<String>,
}
