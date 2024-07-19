use std::str::FromStr;

use discord_rich_presence::{activity::{Activity, Assets, Timestamps}, DiscordIpc, DiscordIpcClient};
use discord_rich_presence::activity::Button as ActButton;
use jellyfin::{Button, EndTime, Item, MediaType, RawSession, Session};
use url::{ParseError, Url};

mod jellyfin;
mod external;

pub(crate) type JfResult<T> = Result<T, Box<dyn std::error::Error>>;

pub struct Client {
    discord_ipc_client: DiscordIpcClient,
    url: Url,
    api_key: String,
    usernames: Vec<String>,
    reqwest: reqwest::Client,
    session: Option<Session>,
    buttons: Option<Vec<Button>>,
    episode_display_options: EpisodeDisplayOptions,
    music_display_options: MusicDisplayOptions,
    blacklist: Blacklist,
    show_paused: bool,
}

impl Client {
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    pub async fn connect(&mut self) -> JfResult<()> {
        self.discord_ipc_client.connect()
    }

    pub async fn reconnect(&mut self) -> JfResult<()> {
        self.discord_ipc_client.reconnect()
    }

    async fn get_session(&mut self) -> Result<(), reqwest::Error> {
        let sessions: Vec<RawSession> = self.reqwest.get(
            format!(
                "{}Sessions?api_key={}",
                self.url,
                self.api_key
            ))
            .send()
            .await?
            .json()
            .await?;

        for session in sessions {
            if self.usernames.iter().all(|u| session.user_name.to_lowercase() != *u) {
                continue;
            }

            if let None = session.now_playing_item {
                continue;
            }

            self.session = Some(session.build());
            return Ok(());
        }
        self.session = None;
        Ok(())
    }

    pub async fn set_activity(&mut self) -> JfResult<()> {
        self.get_session().await?;

        if let Some(session) = &self.session {
            if session.now_playing_item.media_type == MediaType::None {
                eprintln!("Unrecognized media type, returning...");
                return Ok(())
            }

            if self.check_blacklist().await? {
                eprintln!("Content is in blacklist, returning...");
                return Ok(())
            }

            let mut activity = Activity::new();

            let mut image_url = Url::from_str("https://i.imgur.com/oX6vcds.png")?;

            if session.now_playing_item.media_type == MediaType::LiveTv {
                //TODO: Add LiveTv image "https://i.imgur.com/XxdHOqm.png" and turn if/else to if/else if
                image_url = Url::from_str("https://i.imgur.com/XxdHOqm.png")?;
            } else if let Ok(iu) = self.get_image() {
                image_url = iu;
            }

            let mut assets = Assets::new()
                .large_image(image_url.as_str());

            let mut timestamps = Timestamps::new();

            match session.get_endtime()? {
                EndTime::Some(end) => timestamps = timestamps.end(end),
                EndTime::NoEndTime => (),
                EndTime::Paused if self.show_paused => {
                    assets = assets
                        .small_image("https://i.imgur.com/wlHSvYy.png")
                        .small_text("Paused");
                },
                EndTime::Paused => return Ok(()),
            }

            let buttons: Vec<Button>;

            if let Some(b) = self.get_buttons() {
                // This gets around the value being dropped immediately at the end of this if statement
                buttons = b;
                activity = activity.buttons(buttons.iter().map(|b| ActButton::new(&b.name, &b.url)).collect());
            }

            let mut state = self.get_state();

            if state.len() > 128 {
                state = state.chars().take(128).collect();
            } else if state.len() < 3 {
                state += "‎‎";
            }

            let mut details = session.get_details().to_string();

            if details.len() > 128 {
                details = details.chars().take(128).collect();
            } else if details.len() < 3 {
                details += "‎‎";
            }

            activity = activity
                .timestamps(timestamps)
                .assets(assets)
                .details(&details)
                .state(&state);

            self.discord_ipc_client.set_activity(activity)?;
        }
        Ok(())
    }

    pub fn get_buttons(&self) -> Option<Vec<Button>> {
        let session = self.session.as_ref()?;

        let mut activity_buttons: Vec<Button> = Vec::new();

        if let (Some(ext_urls), Some(buttons))
            = (&session.now_playing_item.external_urls, self.buttons.as_ref()) {
            let mut i = 0;
            for button in buttons {
                if activity_buttons.len() == 2 {
                    break
                }

                if button.is_dynamic() {
                    activity_buttons.push(Button::new(ext_urls[i].name.clone(), ext_urls[i].url.clone()));
                    i += 1;
                } else {
                    activity_buttons.push(button.clone())
                }
            }
            return Some(activity_buttons)
        } else if let Some(ext_urls) = &session.now_playing_item.external_urls {
            for ext_url in ext_urls {
                if activity_buttons.len() == 2 {
                    break
                }

                activity_buttons.push(Button::new(ext_url.name.clone(), ext_url.url.clone()))
            }
            return Some(activity_buttons)
        } else if let Some(buttons) = self.buttons.as_ref() {
            for button in buttons {
                if activity_buttons.len() == 2 {
                    break
                }

                if !button.is_dynamic() {
                    activity_buttons.push(button.clone())
                }
            }
            return Some(activity_buttons)
        }
        None
    }

    pub fn get_image(&self) -> Result<Url, ParseError> {
        let session = self.session.as_ref().unwrap();

        match session.now_playing_item.media_type {
            MediaType::Episode => {
                let path = "Items/".to_string() 
                    + session.now_playing_item.series_id.as_ref()
                        .unwrap_or(&session.now_playing_item.id) 
                    + "/Images/Primary";

                self.url.join(&path)
            },
            MediaType::Music => {
                let path = "Items/".to_string() 
                    + session.now_playing_item.album_id.as_ref()
                        .unwrap_or(&session.now_playing_item.id) 
                    + "/Images/Primary";

                self.url.join(&path)
            },
            _ => {
                let path = "Items/".to_string() + &session.now_playing_item.id + "/Images/Primary";

                self.url.join(&path)
            }
        }
    }

    pub fn get_state(&self) -> String {
        let session = self.session.as_ref().unwrap();

        match session.now_playing_item.media_type {
            MediaType::Episode => {
                let episode = (session.now_playing_item.index_number.unwrap_or(0), session.now_playing_item.index_number_end);
                let mut state = String::new();

                if let Some(season) = session.now_playing_item.parent_index_number {
                    if self.episode_display_options.prefix {
                        state += &format!("S{:02}", season);
                    } else {
                        state += &format!("S{}", season);
                    }
                }

                if !state.is_empty() && self.episode_display_options.divider {
                    state += " - "
                }

                if let (first, Some(last)) = episode {
                    if self.episode_display_options.prefix {
                        state += &format!("E{:02} - {:02}", first, last)
                    } else {
                        state += &format!("E{} - {}", first, last)
                    }
                } else {
                    let (episode, _) = episode;
                    if self.episode_display_options.prefix {
                        state += &format!("E{:02}", episode)
                    } else {
                        state += &format!("E{}", episode)
                    }
                }
                
                if !self.episode_display_options.simple {
                    state += &format!(" {}", session.now_playing_item.name)
                }

                state
            },
            MediaType::LiveTv => "Live TV".to_string(),
            MediaType::Music => {
                let mut state = String::new();

                let artists = session.format_artists();
                
                if !artists.is_empty() {
                    state += &format!("By {}", artists)
                }
                
                for data in &self.music_display_options.display {
                    match data.as_str() {
                        "genres" => {
                            let genres = session.now_playing_item.genres
                                .as_ref()
                                .unwrap_or(&vec!["".to_string()])
                                .join(", ");
                            if !state.is_empty() && !genres.is_empty() {
                                state += &format!(" {} ", self.music_display_options.separator);
                            }
                            state += &genres
                        },
                        "year" => {
                            if let Some(year) = session.now_playing_item.production_year {
                                if !state.is_empty() {
                                    state += &format!(" {} ", self.music_display_options.separator);
                                }

                                state += &year.to_string();
                            }
                        },
                        "album" => {
                            if let Some(album) = &session.now_playing_item.album {
                                if !state.is_empty() {
                                    state += &format!(" {} ", self.music_display_options.separator);
                                }

                                state += album;
                            }
                        }
                        _ => ()
                    }
                }

                state
            },
            MediaType::Book => {
                let mut state = String::new();

                if let Some(position_ticks) = session.play_state.position_ticks {
                    let ticks_to_pages = 10000;

                    let page = position_ticks / ticks_to_pages;

                    state += &format!("Reading page {}", page);
                }
                
                state
            },
            MediaType::AudioBook => {
                let mut state = String::new();

                let artists = session.format_artists();

                let genres = session.now_playing_item.genres
                    .as_ref()
                    .unwrap_or(&vec!["".to_string()])
                    .join(", ");
                
                if !artists.is_empty() {
                    state += &format!("By {}", artists)
                }

                if !state.is_empty() && !genres.is_empty() {
                    state += " - "
                }

                state += &genres;

                state
            },
            _ => session.now_playing_item.genres.as_ref().unwrap_or(&vec!["".to_string()]).join(", ")
        }
    }

    async fn get_ancestors(&self) -> JfResult<Vec<Item>> {
        let session = self.session.as_ref().unwrap();

        let ancestors: Vec<Item> = self.reqwest.get(self.url.join(&format!("Items/{}/Ancestors?api_key={}", session.now_playing_item.id, self.api_key))?)
            .send()
            .await?
            .json()
            .await?;

        Ok(ancestors)
    }

    async fn check_blacklist(&self) -> JfResult<bool> {
        let session = self.session.as_ref().unwrap();
        let ancestors = self.get_ancestors().await?;

        if self.blacklist.media_types.iter().any(|m| m == &session.now_playing_item.media_type) {
            return Ok(true)
        }

        if self.blacklist.libraries.iter().any(|l| ancestors.iter().any(|a| l == &a.name)) {
            return Ok(true)
        }
        
        Ok(false)
    }
}

struct EpisodeDisplayOptions {
    divider: bool,
    prefix: bool,
    simple: bool,
}

struct MusicDisplayOptions {
    separator: String,
    display: Vec<String>,
}

struct Blacklist {
    media_types: Vec<MediaType>,
    libraries: Vec<String>,
}

#[derive(Default)]
pub struct ClientBuilder {
    url: String,
    client_id: String,
    api_key: String,
    self_signed: bool,
    usernames: Vec<String>,
    buttons: Option<Vec<Button>>,
    episode_divider: bool,
    episode_prefix: bool,
    episode_simple: bool,
    music_separator: String,
    music_display: Vec<String>,
    blacklist_media_types: Vec<MediaType>,
    blacklist_libraries: Vec<String>,
    show_paused: bool,
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self {
            client_id: "1053747938519679018".to_string(),
            music_separator: "-".to_string(),
            music_display: vec!["genres".to_string()],
            show_paused: true,
            ..Default::default()
        }
    }

    pub fn url<T: Into<String>>(mut self, url: T) -> Self {
        self.url = url.into();
        self
    }

    pub fn client_id<T: Into<String>>(mut self, client_id: T) -> Self {
        self.client_id = client_id.into();
        self
    }

    pub fn api_key<T: Into<String>>(mut self, api_key: T) -> Self {
        self.api_key = api_key.into();
        self
    }

    pub fn self_signed(mut self, self_signed: bool) -> Self {
        self.self_signed = self_signed;
        self
    }

    pub fn usernames(mut self, usernames: Vec<String>) -> Self {
        self.usernames = usernames;
        self
    }

    pub fn username<T: Into<String>>(mut self, username: T) -> Self {
        self.usernames = vec![username.into()];
        self
    }

    pub fn buttons(mut self, buttons: Vec<Button>) -> Self {
        self.buttons = Some(buttons);
        self
    }

    pub fn episode_divider(mut self, val: bool) -> Self {
        self.episode_divider = val;
        self
    }

    pub fn episode_prefix(mut self, val: bool) -> Self {
        self.episode_prefix = val;
        self
    }

    pub fn episode_simple(mut self, val: bool) -> Self {
        self.episode_simple = val;
        self
    }

    pub fn music_separator<T: Into<String>>(mut self, separator: T) -> Self {
        self.music_separator = separator.into();
        self
    }

    pub fn music_display(mut self, display: Vec<String>) -> Self {
        self.music_display = display;
        self
    }

    pub fn blacklist_media_types(mut self, media_types: Vec<MediaType>) -> Self {
        self.blacklist_media_types = media_types;
        self
    }

    pub fn blacklist_libraries(mut self, libraries: Vec<String>) -> Self {
        self.blacklist_libraries = libraries;
        self
    }

    pub fn show_paused(mut self, val: bool) -> Self {
        self.show_paused = val;
        self
    }

    pub fn build(self) -> JfResult<Client> {
        Ok(Client {
            discord_ipc_client: DiscordIpcClient::new(&self.client_id)?,
            url: self.url.parse()?,
            api_key: self.api_key,
            reqwest: reqwest::Client::builder().danger_accept_invalid_certs(self.self_signed).build()?,
            usernames: self.usernames,
            buttons: self.buttons,
            session: None,
            episode_display_options: EpisodeDisplayOptions {
                divider: self.episode_divider,
                prefix: self.episode_prefix,
                simple: self.episode_simple,
            },
            music_display_options: MusicDisplayOptions {
                separator: self.music_separator,
                display: self.music_display,
            },
            blacklist: Blacklist {
                media_types: self.blacklist_media_types,
                libraries: self.blacklist_libraries,
            },
            show_paused: self.show_paused,
        })
    }
}
