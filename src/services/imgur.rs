use serde_json::Value;
use std::env;
use std::io::Write;

/*
    TODO: Comments
*/

macro_rules! imgur_api (
    ($url: expr) => (
        concat!("https://api.imgur.com/3/", $url)
    );
);

#[derive(Default)]
pub struct Imgur {
    pub url: String,
}

#[derive(Debug)]
pub enum ImgurError {
    Reqwest(String),
    Io(String),
    Json(String),
    VarError(String),
}

impl From<reqwest::Error> for ImgurError {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(format!("Error uploading image: {}", value))
    }
}

impl From<std::io::Error> for ImgurError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(format!("Unable to open file: {}", value))
    }
}

impl From<serde_json::Error> for ImgurError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(format!("Unable to parse urls: {}", value))
    }
}

impl From<env::VarError> for ImgurError {
    fn from(value: env::VarError) -> Self {
        Self::VarError(format!("Unable to get environment variables: {}", value))
    }
}

pub fn get_urls_path() -> Result<String, ImgurError> {
    if cfg!(not(windows)) {
        let user = env::var("USER")?;
        if user != "root" {
            let xdg_config_home = env::var("XDG_CONFIG_HOME")
                .unwrap_or_else(|_| env::var("HOME").unwrap() + "/.config");
            Ok(xdg_config_home + ("/jellyfin-rpc/urls.json"))
        } else {
            Ok("/etc/jellyfin-rpc/urls.json".to_string())
        }
    } else {
        let app_data = env::var("APPDATA")?;
        Ok(app_data + r"\jellyfin-rpc\urls.json")
    }
}

impl Imgur {
    pub async fn get(
        image_url: &String,
        item_id: &String,
        client_id: &String,
        image_urls_file: Option<String>,
    ) -> Result<Self, ImgurError> {
        let file = image_urls_file.unwrap_or_else(|| get_urls_path().unwrap());
        let mut json = Imgur::read_file(file.clone())?;
        if let Some(value) = json.get(item_id).and_then(Value::as_str) {
            return Ok(Self {
                url: value.to_string(),
            });
        }

        Ok(Self {
            url: Imgur::write_file(file, image_url, item_id, client_id, &mut json).await?,
        })
    }

    fn read_file(file: String) -> Result<Value, ImgurError> {
        let content = std::fs::read_to_string(file.clone()).unwrap_or_else(|_| {
            std::fs::create_dir_all(std::path::Path::new(&file).parent().unwrap()).ok();
            std::fs::File::create(file.clone())
                .map(|mut file| {
                    write!(file, "{{\n}}").ok();
                    file
                })
                .unwrap();
            std::fs::read_to_string(file).unwrap()
        });
        let json: Value = serde_json::from_str(&content)?;
        Ok(json)
    }

    async fn write_file(
        file: String,
        image_url: &String,
        item_id: &String,
        client_id: &String,
        json: &mut Value,
    ) -> Result<String, ImgurError> {
        let mut new_data = serde_json::Map::new();
        let imgur_url = Imgur::upload(image_url, client_id).await?;
        new_data.insert(item_id.to_string(), serde_json::json!(imgur_url));

        let data = json.as_object_mut().unwrap();
        data.append(&mut new_data);

        write!(std::fs::File::create(file)?, "{}", serde_json::json!(data))?;
        Ok(imgur_url)
    }

    async fn upload(image_url: &String, client_id: &String) -> Result<String, ImgurError> {
        let img = reqwest::get(image_url).await?.bytes().await?;
        let client = reqwest::Client::new();
        let response = client
            .post(imgur_api!("image"))
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Client-ID {}", client_id),
            )
            .body(img)
            .send()
            .await?;
        let val: Value = serde_json::from_str(&response.text().await?)?;

        Ok(val["data"]["link"].as_str().unwrap().to_string())
    }
}
