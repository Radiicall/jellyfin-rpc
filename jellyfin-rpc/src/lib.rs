use discord_rich_presence::{DiscordIpc, DiscordIpcClient};
use jellyfin::Session;
use url::Url;

mod jellyfin;

type JfResult<T> = Result<T, Box<dyn std::error::Error>>;

pub struct Client {
    discord_ipc_client: DiscordIpcClient,
    url: Url,
    api_key: String,
    usernames: Vec<String>,
    reqwest: reqwest::Client,
}

impl Client {
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    pub async fn connect(&mut self) -> JfResult<()> {
        self.discord_ipc_client.connect()
    }

    async fn get_session(&self) -> Result<Option<Session>, reqwest::Error> {
        let sessions: Vec<Session> = self.reqwest.get(
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

            return Ok(Some(session));
        }
        Ok(None)
    }

    pub async fn set_activity(&self) {
        let session = self.get_session().await.unwrap();

        if let Some(session) = session {
            match session.now_playing_item().media_type {
                jellyfin::MediaType::Movie => {},
                jellyfin::MediaType::Episode => {},
                jellyfin::MediaType::LiveTv => {},
                jellyfin::MediaType::Music => {},
                jellyfin::MediaType::Book => {},
                jellyfin::MediaType::AudioBook => {},
                jellyfin::MediaType::None => (),
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
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self {
            url: "http://example.com".to_string(),
            client_id: "1053747938519679018".to_string(),
            api_key: "placeholder".to_string(),
            self_signed: false,
            usernames: vec![]
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

    pub fn usernames(mut self, username: Option<String>, usernames: Option<Vec<String>>) -> Self {
        if let Some(username) = username {
            self.usernames = vec![username]
        } else if let Some(usernames) = usernames {
            self.usernames = usernames
        } else {
            eprintln!("usernames function called but nothing was provided!")
        }

        self
    }

    pub fn build(self) -> JfResult<Client> {
        Ok(Client {
            discord_ipc_client: DiscordIpcClient::new(&self.client_id)?,
            url: self.url.parse()?,
            api_key: self.api_key,
            reqwest: reqwest::Client::builder().danger_accept_invalid_certs(self.self_signed).build()?,
            usernames: self.usernames,
        })
    }
}
