use discord_rich_presence::activity::{ActivityType, Button as ActButton};
use discord_rich_presence::{
    activity::{Activity, Assets, Timestamps},
    DiscordIpc, DiscordIpcClient,
};
pub use error::JfError;
pub use jellyfin::{Button, MediaType};
use jellyfin::{ExternalUrl, Item, PlayTime, RawSession, Session};
use log::debug;
use reqwest::header::{HeaderMap, AUTHORIZATION};
use std::str::FromStr;
use url::Url;

mod error;
mod external;
mod jellyfin;
#[cfg(test)]
mod tests;

pub(crate) type JfResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Client used to interact with jellyfin and discord
pub struct Client {
    discord_ipc_client: DiscordIpcClient,
    url: Url,
    usernames: Vec<String>,
    reqwest: reqwest::blocking::Client,
    session: Option<Session>,
    buttons: Option<Vec<Button>>,
    episode_display_options: EpisodeDisplayOptions,
    music_display_options: DisplayOptions,
    movies_display_options: DisplayOptions,
    blacklist: Blacklist,
    show_paused: bool,
    show_images: bool,
    imgur_options: ImgurOptions,
}

impl Client {
    /// Calls the `ClientBuilder::new()` function
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    /// Connects to the discord socket
    pub fn connect(&mut self) -> JfResult<()> {
        self.discord_ipc_client.connect()
    }

    /// Reconnects to the discord socket
    pub fn reconnect(&mut self) -> JfResult<()> {
        self.discord_ipc_client.reconnect()
    }

    /// Clears current activity on discord if anything is being displayed
    ///
    /// # Example
    /// ```no_run
    /// use jellyfin_rpc::Client;
    ///
    /// let mut builder = Client::builder();
    /// builder.api_key("abcd1234")
    ///     .url("https://jellyfin.example.com")
    ///     .username("user");
    ///
    /// let mut client = builder.build().unwrap();
    ///
    /// client.connect().unwrap();
    ///
    /// client.set_activity().unwrap();
    ///
    /// client.clear_activity().unwrap();
    /// ```
    pub fn clear_activity(&mut self) -> JfResult<()> {
        self.discord_ipc_client.clear_activity()
    }

    /// Gathers information from jellyfin about what is being played and displays it according to the options supplied to the builder.
    ///
    /// # Example
    /// ```no_run
    /// use jellyfin_rpc::Client;
    ///
    /// let mut builder = Client::builder();
    /// builder.api_key("abcd1234")
    ///     .url("https://jellyfin.example.com")
    ///     .username("user");
    ///
    /// let mut client = builder.build().unwrap();
    ///
    /// client.connect().unwrap();
    ///
    /// client.set_activity().unwrap();
    /// ```
    pub fn set_activity(&mut self) -> JfResult<String> {
        self.get_session()?;

        if let Some(session) = &self.session {
            if session.now_playing_item.media_type == MediaType::None {
                return Err(Box::new(JfError::UnrecognizedMediaType));
            }

            if self.check_blacklist()? {
                return Err(Box::new(JfError::ContentBlacklist));
            }

            let mut activity = Activity::new();

            let activity_text = self.get_activity_text();

            let mut image_url = Url::from_str("https://i.imgur.com/oX6vcds.png")?;

            if session.now_playing_item.media_type == MediaType::LiveTv {
                image_url = Url::from_str("https://i.imgur.com/XxdHOqm.png")?;
            } else if self.imgur_options.enabled && self.show_images {
                if let Ok(imgur_url) = external::imgur::get_image(self) {
                    image_url = imgur_url;
                } else {
                    debug!("imgur::get_image() didnt return an image, using default..")
                }
            } else if self.show_images {
                if let Ok(iu) = self.get_image() {
                    image_url = iu;
                } else {
                    debug!("self.get_image() didnt return an image, using default..")
                }
            }

            let mut assets = Assets::new().large_image(image_url.as_str());

            let image_text = activity_text.image_text;

            if !image_text.is_empty() {
                assets = assets.large_text(&image_text);
            }

            let mut timestamps = Timestamps::new();

            match session.get_time()? {
                PlayTime::Some(start, end) => timestamps = timestamps.start(start).end(end),
                PlayTime::None => (),
                PlayTime::Paused if self.show_paused => {
                    assets = assets
                        .small_image("https://i.imgur.com/wlHSvYy.png")
                        .small_text("Paused");
                }
                PlayTime::Paused => return Ok(String::new()),
            }

            let buttons: Vec<Button>;

            if let Some(b) = self.get_buttons() {
                // This gets around the value being dropped immediately at the end of this if statement
                buttons = b;
                activity = activity.buttons(
                    buttons
                        .iter()
                        .map(|b| ActButton::new(&b.name, &b.url))
                        .collect(),
                );
            }

            let mut state_text = activity_text.state_text;

            if state_text.len() > 128 {
                state_text = state_text.chars().take(128).collect();
            }

            let mut details_text = activity_text.details_text;

            if details_text.len() > 128 {
                details_text = details_text.chars().take(128).collect();
            }

            match session.now_playing_item.media_type {
                MediaType::Book => (),
                MediaType::Music | MediaType::AudioBook => {
                    activity = activity.activity_type(ActivityType::Listening)
                }
                _ => activity = activity.activity_type(ActivityType::Watching),
            }

            activity = activity
                .timestamps(timestamps)
                .assets(assets)
                .state(&state_text)
                .details(&details_text);

            self.discord_ipc_client.set_activity(activity)?;

            return Ok(format!("{} | {}", state_text, details_text));
        }
        Ok(String::new())
    }

    fn get_session(&mut self) -> Result<(), reqwest::Error> {
        let sessions: Vec<RawSession> = self
            .reqwest
            .get(format!("{}Sessions", self.url))
            .send()?
            .json()?;

        debug!("Found {} sessions", sessions.len());

        for session in sessions {
            debug!("Session username is {:?}", session.user_name);
            if let Some(username) = session.user_name.as_ref() {
                if self
                    .usernames
                    .iter()
                    .all(|u| username.to_lowercase() != u.to_lowercase())
                {
                    continue;
                }

                if session.now_playing_item.is_none() {
                    continue;
                }
                debug!("NowPlayingItem exists");

                if session.play_state.is_none() {
                    continue;
                }
                debug!("PlayState exists");

                let session = session.build();

                if session
                    .now_playing_item
                    .extra_type
                    .as_ref()
                    .is_some_and(|et| et == "ThemeSong")
                {
                    debug!("Session is playing a theme song, continuing loop");
                    continue;
                }

                self.session = Some(session);
                return Ok(());
            }
        }
        self.session = None;
        Ok(())
    }

    fn get_buttons(&self) -> Option<Vec<Button>> {
        let session = self.session.as_ref()?;

        let mut activity_buttons: Vec<Button> = Vec::new();

        if let (Some(ext_urls), Some(buttons)) = (
            &session.now_playing_item.external_urls,
            self.buttons.as_ref(),
        ) {
            let ext_urls: Vec<&ExternalUrl> = ext_urls
                .iter()
                .filter(|eu| {
                    !eu.url.starts_with("http://localhost")
                        && !eu.url.starts_with("https://localhost")
                })
                .collect();
            let mut i = 0;
            for button in buttons {
                if activity_buttons.len() == 2 {
                    break;
                }

                if button.is_dynamic() {
                    if ext_urls.len() > i {
                        activity_buttons.push(Button::new(
                            ext_urls[i].name.clone(),
                            ext_urls[i].url.clone(),
                        ));
                        i += 1;
                    }
                } else {
                    activity_buttons.push(button.clone())
                }
            }
            return Some(activity_buttons);
        } else if let Some(buttons) = self.buttons.as_ref() {
            for button in buttons {
                if activity_buttons.len() == 2 {
                    break;
                }

                if !button.is_dynamic() {
                    activity_buttons.push(button.clone())
                }
            }
            return Some(activity_buttons);
        } else if let Some(ext_urls) = &session.now_playing_item.external_urls {
            let ext_urls: Vec<&ExternalUrl> = ext_urls
                .iter()
                .filter(|eu| {
                    !eu.url.starts_with("http://localhost")
                        && !eu.url.starts_with("https://localhost")
                })
                .collect();
            for ext_url in ext_urls {
                if activity_buttons.len() == 2 {
                    break;
                }

                activity_buttons.push(Button::new(ext_url.name.clone(), ext_url.url.clone()))
            }
            return Some(activity_buttons);
        }
        None
    }

    fn get_image(&self) -> JfResult<Url> {
        let session = self.session.as_ref().unwrap();

        let path = "Items/".to_string() + &session.item_id + "/Images/Primary";

        let image_url = self.url.join(&path)?;

        if self
            .reqwest
            .get(image_url.as_ref())
            .send()?
            .text()?
            .contains("does not have an image of type Primary")
        {
            Err(Box::new(JfError::NoImage))
        } else {
            Ok(image_url)
        }
    }

    fn get_activity_text(&self) -> DisplayText {
        let session = self.session.as_ref().unwrap();

        match session.now_playing_item.media_type {
            MediaType::Episode => {
                let episode = (
                    session.now_playing_item.index_number.unwrap_or(0),
                    session.now_playing_item.index_number_end,
                );
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

                DisplayText {
                    image_text: String::new(),
                    state_text: state,
                    details_text: String::new(),
                }
            }
            MediaType::LiveTv => DisplayText {
                image_text: String::new(),
                state_text: "Live TV".to_string(),
                details_text: String::new(),
            },
            MediaType::Music => {
                let artists = session.format_artists();

                let title = &session.now_playing_item.name;

                let genres = session
                    .now_playing_item
                    .genres
                    .as_ref()
                    .unwrap_or(&vec!["".to_string()])
                    .join(", ");

                let year = session
                    .now_playing_item
                    .production_year
                    .map(|x| x.to_string())
                    .unwrap_or(String::new());

                let album = session
                    .now_playing_item
                    .album
                    .clone()
                    .unwrap_or(String::new());

                let image_text = self
                    .music_display_options
                    .image_text
                    .replace("{title}", &title)
                    .replace("{artists}", &artists)
                    .replace("{genres}", &genres)
                    .replace("{year}", &year)
                    .replace("{album}", &album);

                let state_text = self
                    .music_display_options
                    .state_text
                    .replace("{title}", &title)
                    .replace("{artists}", &artists)
                    .replace("{genres}", &genres)
                    .replace("{year}", &year)
                    .replace("{album}", &album);

                let details_text = self
                    .music_display_options
                    .details_text
                    .replace("{title}", &title)
                    .replace("{artists}", &artists)
                    .replace("{genres}", &genres)
                    .replace("{year}", &year)
                    .replace("{album}", &album);

                DisplayText {
                    state_text,
                    details_text,
                    image_text,
                }
            }
            MediaType::Book => {
                let mut state = String::new();

                if let Some(position_ticks) = session.play_state.position_ticks {
                    let ticks_to_pages = 10000;

                    let page = position_ticks / ticks_to_pages;

                    state += &format!("Reading page {}", page);
                }

                DisplayText {
                    image_text: String::new(),
                    state_text: state,
                    details_text: String::new(),
                }
            }
            MediaType::AudioBook => {
                let mut state = String::new();

                let artists = session.format_artists();

                let genres = session
                    .now_playing_item
                    .genres
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

                DisplayText {
                    image_text: String::new(),
                    state_text: state,
                    details_text: String::new(),
                }
            }
            MediaType::Movie => {
                let genres = session
                    .now_playing_item
                    .genres
                    .as_ref()
                    .unwrap_or(&vec!["".to_string()])
                    .join(", ");

                let year = session
                    .now_playing_item
                    .production_year
                    .map(|x| x.to_string())
                    .unwrap_or(String::new());

                let critic_rating = session
                    .now_playing_item
                    .critic_rating
                    .map(|x| x.to_string())
                    .unwrap_or(String::new());

                let community_rating = session
                    .now_playing_item
                    .community_rating
                    .map(|x| x.to_string())
                    .unwrap_or(String::new());

                let image_text = self
                    .music_display_options
                    .image_text
                    .replace("{genres}", &genres)
                    .replace("{year}", &year)
                    .replace("{critic_rating}", &critic_rating)
                    .replace("{community_rating}", &community_rating);

                let details_line_1 = self
                    .music_display_options
                    .state_text
                    .replace("{genres}", &genres)
                    .replace("{year}", &year)
                    .replace("{critic_rating}", &critic_rating)
                    .replace("{community_rating}", &community_rating);

                let details_line_2 = self
                    .music_display_options
                    .details_text
                    .replace("{genres}", &genres)
                    .replace("{year}", &year)
                    .replace("{critic_rating}", &critic_rating)
                    .replace("{community_rating}", &community_rating);

                DisplayText {
                    image_text,
                    state_text: details_line_1,
                    details_text: details_line_2,
                }
            }
            _ => {
                let state = session
                    .now_playing_item
                    .genres
                    .as_ref()
                    .unwrap_or(&vec!["".to_string()])
                    .join(", ");

                DisplayText {
                    image_text: String::new(),
                    state_text: state,
                    details_text: String::new(),
                }
            }
        }
    }

    fn get_ancestors(&self) -> JfResult<Vec<Item>> {
        let session = self.session.as_ref().unwrap();

        let ancestors: Vec<Item> = self
            .reqwest
            .get(
                self.url
                    .join(&format!("Items/{}/Ancestors", session.now_playing_item.id))?,
            )
            .send()?
            .json()?;

        debug!("Ancestors: {:?}", ancestors);

        Ok(ancestors)
    }

    fn check_blacklist(&self) -> JfResult<bool> {
        let session = self.session.as_ref().unwrap();
        let ancestors = self.get_ancestors()?;

        if self
            .blacklist
            .media_types
            .iter()
            .any(|m| m == &session.now_playing_item.media_type)
        {
            return Ok(true);
        }

        if self.blacklist.libraries.iter().any(|l| {
            ancestors
                .iter()
                .any(|a| l == a.name.as_ref().unwrap_or(&"".to_string()))
        }) {
            return Ok(true);
        }

        Ok(false)
    }
}

struct EpisodeDisplayOptions {
    divider: bool,
    prefix: bool,
    simple: bool,
}

struct DisplayOptions {
    image_text: String,
    state_text: String,
    details_text: String,
}

struct DisplayText {
    image_text: String,
    state_text: String,
    details_text: String,
}

struct Blacklist {
    media_types: Vec<MediaType>,
    libraries: Vec<String>,
}

struct ImgurOptions {
    enabled: bool,
    client_id: String,
    urls_location: String,
}

/// Used to build a new Client
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
    music_image_text: String,
    music_state_text: String,
    music_details_text: String,
    movie_image_text: String,
    movie_state_text: String,
    movie_details_text: String,
    blacklist_media_types: Vec<MediaType>,
    blacklist_libraries: Vec<String>,
    show_paused: bool,
    show_images: bool,
    use_imgur: bool,
    imgur_client_id: String,
    imgur_urls_file_location: String,
    large_image_text: String,
}

impl ClientBuilder {
    /// Returns a ClientBuilder with some default options set
    pub fn new() -> Self {
        Self {
            client_id: "1053747938519679018".to_string(),
            music_image_text: "{track}".to_string(),
            music_state_text: "by {artists}".to_string(),
            music_details_text: "on {album} ({genres})".to_string(),
            movie_image_text: "{title}".to_string(),
            movie_state_text: "{episode_title}".to_string(),
            movie_details_text: "E{episode_number}S{season_number}".to_string(),
            show_paused: true,
            ..Default::default()
        }
    }

    /// Jellyfin URL to be used by the client.
    ///
    /// Has no default.
    pub fn url<T: Into<String>>(&mut self, url: T) -> &mut Self {
        self.url = url.into();
        self
    }

    /// Discord Application ID that the client will use when connecting to Discord.
    ///
    /// Defaults to `"1053747938519679018"`.
    pub fn client_id<T: Into<String>>(&mut self, client_id: T) -> &mut Self {
        self.client_id = client_id.into();
        self
    }

    /// Jellyfin API Key that will be used to gather data about what is being played.
    ///
    /// Has no default.
    pub fn api_key<T: Into<String>>(&mut self, api_key: T) -> &mut Self {
        self.api_key = api_key.into();
        self
    }

    /// Controls the use of certificate validation in reqwest.
    ///
    /// Defaults to `false`.
    pub fn self_signed(&mut self, self_signed: bool) -> &mut Self {
        self.self_signed = self_signed;
        self
    }

    /// Usernames that should be matched when checking Jellyfin sessions.
    ///
    /// Has no default.
    ///
    /// # Warning
    /// This overwrites the value set in `ClientBuilder::Username()`,
    /// only one of these 2 should be used
    pub fn usernames(&mut self, usernames: Vec<String>) -> &mut Self {
        self.usernames = usernames;
        self
    }

    /// same as `ClientBuilder::Usernames()` but will only accept a single username
    ///
    /// Has no default.
    ///
    /// # Warning
    /// This overwrites the value set in `ClientBuilder::Usernames()`,
    /// only one of these 2 should be used
    pub fn username<T: Into<String>>(&mut self, username: T) -> &mut Self {
        self.usernames = vec![username.into()];
        self
    }

    /// buttons to be displayed on the activity.
    /// Pass an empty `Vec::new()` to display no buttons
    ///
    /// Defaults to dynamic buttons generated from the Jellyfin session.
    pub fn buttons(&mut self, buttons: Vec<Button>) -> &mut Self {
        self.buttons = Some(buttons);
        self
    }

    /// Splits season and episode numbers with a dash.
    ///
    /// Defaults to `false`.
    ///
    /// # Example
    /// S1E1 Pilot -> S1 - E1 Pilot
    pub fn episode_divider(&mut self, val: bool) -> &mut Self {
        self.episode_divider = val;
        self
    }

    /// Adds leading 0's to season and episode numbers.
    ///
    /// Defaults to `false`.
    ///
    /// # Example
    /// S1E1 Pilot -> S01E01 Pilot
    pub fn episode_prefix(&mut self, val: bool) -> &mut Self {
        self.episode_prefix = val;
        self
    }

    /// Removes the episode name from the activity.
    ///
    /// Defaults to `false`.
    ///
    /// # Example
    /// S1E1 Pilot -> S1E1
    pub fn episode_simple(&mut self, val: bool) -> &mut Self {
        self.episode_simple = val;
        self
    }

    pub fn music_state_text(&mut self, state_text: String) -> &mut Self {
        self.music_state_text = state_text;
        self
    }

    pub fn music_details_text(&mut self, details_text: String) -> &mut Self {
        self.music_details_text = details_text;
        self
    }

    pub fn music_image_text(&mut self, image_text: String) -> &mut Self {
        self.music_image_text = image_text;
        self
    }

    pub fn movies_state_text(&mut self, state_text: String) -> &mut Self {
        self.music_state_text = state_text;
        self
    }

    pub fn movies_details_text(&mut self, details_text: String) -> &mut Self {
        self.music_details_text = details_text;
        self
    }

    pub fn movies_image_text(&mut self, image_text: String) -> &mut Self {
        self.music_image_text = image_text;
        self
    }

    /// Blacklist certain `MediaType`s so they don't display.
    ///
    /// Defaults to `Vec::new()`.
    pub fn blacklist_media_types(&mut self, media_types: Vec<MediaType>) -> &mut Self {
        self.blacklist_media_types = media_types;
        self
    }

    /// Blacklist certain libraries so they don't display.
    ///
    /// Defaults to `Vec::new()`.
    pub fn blacklist_libraries(&mut self, libraries: Vec<String>) -> &mut Self {
        self.blacklist_libraries = libraries;
        self
    }

    /// Show activity when paused.
    ///
    /// Defaults to `true`.
    pub fn show_paused(&mut self, val: bool) -> &mut Self {
        self.show_paused = val;
        self
    }

    /// Show images from jellyfin on the activity.
    ///
    /// Defaults to `false`.
    pub fn show_images(&mut self, val: bool) -> &mut Self {
        self.show_images = val;
        self
    }

    /// Use imgur for images, uploads images from jellyfin to imgur and stores the imgur links in a local cache
    ///
    /// Defaults to `false`.
    pub fn use_imgur(&mut self, val: bool) -> &mut Self {
        self.use_imgur = val;
        self
    }

    /// Imgur client id, used to upload images through their API.
    ///
    /// Empty by default.
    pub fn imgur_client_id<T: Into<String>>(&mut self, client_id: T) -> &mut Self {
        self.imgur_client_id = client_id.into();
        self
    }

    /// Where to store the URLs to images uploaded to imgur.
    /// Having this cache lets you avoid uploading the same image several times to their service.
    ///
    /// Empty by default.
    ///
    /// # Warning
    /// Setting this to something like `/dev/null` is **NOT** recommended,
    /// jellyfin-rpc will upload the image every time you call `Client::set_activity()`
    /// if it can't find the image its looking for in the cache.
    pub fn imgur_urls_file_location<T: Into<String>>(&mut self, location: T) -> &mut Self {
        self.imgur_urls_file_location = location.into();
        self
    }

    /// Text to be displayed when hovering the large activity image in Discord
    ///
    /// Empty by default
    pub fn large_image_text<T: Into<String>>(&mut self, text: T) -> &mut Self {
        self.large_image_text = text.into();
        self
    }

    /// Builds a client from the options specified in the builder.
    ///
    /// # Example
    /// ```
    /// use jellyfin_rpc::ClientBuilder;
    ///
    /// let mut builder = ClientBuilder::new();
    /// builder.api_key("abcd1234")
    ///     .url("https://jellyfin.example.com")
    ///     .username("user");
    ///
    /// let mut client = builder.build().unwrap();
    /// ```
    pub fn build(self) -> JfResult<Client> {
        if self.url.is_empty() || self.usernames.is_empty() || self.api_key.is_empty() {
            return Err(Box::new(JfError::MissingRequiredValues));
        }

        let mut headers = HeaderMap::new();

        headers.insert(
            AUTHORIZATION,
            format!("MediaBrowser Token=\"{}\"", self.api_key).parse()?,
        );
        headers.insert("X-Emby-Token", self.api_key.parse()?);

        Ok(Client {
            discord_ipc_client: DiscordIpcClient::new(&self.client_id)?,
            url: self.url.parse()?,
            reqwest: reqwest::blocking::Client::builder()
                .default_headers(headers)
                .danger_accept_invalid_certs(self.self_signed)
                .build()?,
            usernames: self.usernames,
            buttons: self.buttons,
            session: None,
            episode_display_options: EpisodeDisplayOptions {
                divider: self.episode_divider,
                prefix: self.episode_prefix,
                simple: self.episode_simple,
            },
            music_display_options: DisplayOptions {
                state_text: self.music_state_text,
                details_text: self.music_details_text,
                image_text: self.music_image_text,
            },
            movies_display_options: DisplayOptions {
                state_text: self.movie_state_text,
                details_text: self.movie_details_text,
                image_text: self.movie_image_text,
            },
            blacklist: Blacklist {
                media_types: self.blacklist_media_types,
                libraries: self.blacklist_libraries,
            },
            show_paused: self.show_paused,
            show_images: self.show_images,
            imgur_options: ImgurOptions {
                enabled: self.use_imgur,
                client_id: self.imgur_client_id,
                urls_location: self.imgur_urls_file_location,
            },
        })
    }
}
