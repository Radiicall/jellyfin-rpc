use std::str::FromStr;

use discord_rich_presence::{activity::{Activity, Assets, Timestamps}, DiscordIpc, DiscordIpcClient};
use discord_rich_presence::activity::Button as ActButton;
use jellyfin::{Button, MediaType, RawSession, Session};
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

            let mut activity = Activity::new();

            let mut image_url = Url::from_str("https://i.imgur.com/oX6vcds.png")?;

            if session.now_playing_item.media_type == MediaType::LiveTv {
                //TODO: Add LiveTv image "https://i.imgur.com/XxdHOqm.png" and turn if/else to if/else if
            } else {
                image_url = self.get_image()?;
            }

            if session.play_state.is_paused {
                activity = activity.clone().assets(Assets::new()
                    .large_image(image_url.as_str())
                    .small_image("https://i.imgur.com/wlHSvYy.png")
                    .small_text("Paused"));
            } else {
                activity = activity.assets(Assets::new()
                    .large_image(image_url.as_str()));
            }

            let buttons: Vec<Button>;

            if let Some(b) = self.get_buttons() {
                // This gets around the value being dropped immediately at the end of this if statement
                buttons = b;
                activity = activity.buttons(buttons.iter().map(|b| ActButton::new(&b.name, &b.url)).collect());
            }

            activity = activity.clone()
                .details(session.get_details())
                .state("test")
                .timestamps(Timestamps::new().start(0));

            self.discord_ipc_client.set_activity(activity).unwrap();
        }
        Ok(())
    }

    pub fn get_buttons(&self) -> Option<Vec<Button>> {
        let mut activity_buttons: Vec<Button> = Vec::new();
        if let (Some(ext_urls), Some(buttons))
            = (&self.session.as_ref().unwrap().now_playing_item.external_urls, self.buttons.as_ref()) {
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
        } else if let Some(ext_urls) = &self.session.as_ref().unwrap().now_playing_item.external_urls {
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

    pub fn get_state(&self) -> &str {
        let session = self.session.as_ref().unwrap();

        match session.now_playing_item.media_type {
            MediaType::Episode => {
                let episode = (session.now_playing_item.index_number.unwrap_or(0), session.now_playing_item.index_number_end);
                let mut state = "";

                if let Some(season) = session.now_playing_item.parent_index_number {
                    
                    state = &format!("S{}", season);
                }

                if let (first, Some(last)) = episode {

                } else {

                }
                
                todo!()
            },
            MediaType::LiveTv => "Live TV",
            MediaType::Music => todo!(),
            MediaType::Book => todo!(),
            MediaType::AudioBook => todo!(),
            _ => {
                // I swear this is temporary
                "let genres = self.now_playing_item.genres.as_ref().unwrap_or(&vec![\"\".to_string()]);"
            }
        }
    }
}

pub struct ClientBuilder {
    url: String,
    client_id: String,
    api_key: String,
    self_signed: bool,
    usernames: Vec<String>,
    buttons: Option<Vec<Button>>,
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self {
            url: "http://example.com".to_string(),
            client_id: "1053747938519679018".to_string(),
            api_key: "placeholder".to_string(),
            self_signed: false,
            usernames: vec![],
            buttons: None,
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

    pub fn build(self) -> JfResult<Client> {
        Ok(Client {
            discord_ipc_client: DiscordIpcClient::new(&self.client_id)?,
            url: self.url.parse()?,
            api_key: self.api_key,
            reqwest: reqwest::Client::builder().danger_accept_invalid_certs(self.self_signed).build()?,
            usernames: self.usernames,
            buttons: self.buttons,
            session: None,
        })
    }
}
