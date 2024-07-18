use std::str::FromStr;

use discord_rich_presence::{activity::{Activity, Assets, Timestamps}, DiscordIpc, DiscordIpcClient};
use discord_rich_presence::activity::Button as ActButton;
use jellyfin::{Button, MediaType, RawSession, Session};
use url::Url;

mod jellyfin;
mod external;

pub(crate) type JfResult<T> = Result<T, Box<dyn std::error::Error>>;

pub struct Client {
    discord_ipc_client: DiscordIpcClient,
    url: Url,
    api_key: String,
    usernames: Vec<String>,
    reqwest: reqwest::Client,
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

    async fn get_session(&self) -> Result<Option<Session>, reqwest::Error> {
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

            return Ok(Some(session.build()));
        }
        Ok(None)
    }

    pub async fn set_activity(&mut self) -> JfResult<()> {
        let session = self.get_session().await?;

        if let Some(session) = session {
            let mut activity = Activity::new();

            let mut image_url = Url::from_str("https://i.imgur.com/oX6vcds.png")?;

            if session.now_playing_item.media_type == MediaType::LiveTv {
                //TODO: Add LiveTv image "https://i.imgur.com/XxdHOqm.png" and turn if/else to if/else if
            } else {
                image_url = session.get_image(&self.url)?;
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

            if let Some(b) = session.get_buttons(self.buttons.clone()) {
                // This gets around the value being dropped immediately at the end of this if statement
                buttons = b;
                activity = activity.buttons(buttons.iter().map(|b| ActButton::new(&b.name, &b.url)).collect());
            }

            activity = activity.clone().state("test").details("test").timestamps(Timestamps::new().start(0));

            self.discord_ipc_client.set_activity(activity).unwrap();
        }
        Ok(())
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
        })
    }
}
