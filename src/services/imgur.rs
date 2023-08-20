use crate::core::error::ImgurError;
use serde_json::{json, Value};
use std::io::Write;
use std::{env, fs, path};

/*
    TODO: Comments
*/

#[derive(Default)]
pub struct Imgur {
    pub url: String,
}

pub fn get_urls_path() -> Result<String, ImgurError> {
    if cfg!(not(windows)) {
        let xdg_config_home = env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
            env::var("HOME").expect("No HOME environment variable") + "/.config"
        });
        Ok(xdg_config_home + ("/jellyfin-rpc/urls.json"))
    } else {
        let app_data = env::var("APPDATA")?;
        Ok(app_data + r"\jellyfin-rpc\urls.json")
    }
}

impl Imgur {
    pub async fn get(
        image_url: &str,
        item_id: &str,
        client_id: &str,
        image_urls_file: Option<String>,
    ) -> Result<Self, ImgurError> {
        let file = image_urls_file
            .unwrap_or_else(|| get_urls_path().expect("Failed to get \"urls.json\" file path"));
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
        let content = fs::read_to_string(file.clone()).unwrap_or_else(|_| {
            // Create directories
            let path = path::Path::new(&file).parent().unwrap_or_else(|| {
                eprintln!("Unable to convert \"{}\" to path", file);
                std::process::exit(1);
            });
            fs::create_dir_all(path).ok();

            // Create urls.json file
            fs::File::create(file.clone())
                .map(|mut file| {
                    write!(file, "{{\n}}").ok();
                    file
                })
                .unwrap_or_else(|err| {
                    eprintln!("Unable to create file: \"{}\"\nerror: {}", file, err);
                    std::process::exit(1)
                });

            // Read the newly created file
            fs::read_to_string(file.clone()).unwrap_or_else(|err| {
                eprintln!("Unable to read file: \"{}\"\nerror: {}", file, err);
                std::process::exit(1);
            })
        });

        let json: Value = serde_json::from_str(&content)?;
        Ok(json)
    }

    async fn write_file(
        file: String,
        image_url: &str,
        item_id: &str,
        client_id: &str,
        json: &mut Value,
    ) -> Result<String, ImgurError> {
        // Create a new map that's used for adding data to the "urls.json" file
        let mut new_data = serde_json::Map::new();
        // Upload the content's image to imgur
        let imgur_url = Imgur::upload(image_url, client_id).await?;
        // Insert the item_id and the new image url into the map we created earlier
        new_data.insert(item_id.to_string(), json!(imgur_url));

        // Turn the old json data into a map and append the new map to the old one
        let data = json.as_object_mut().expect("\"urls.json\" file is not an object, try deleting the file and running the program again.");
        data.append(&mut new_data);

        // Overwrite the "urls.json" file with the new data
        write!(fs::File::create(file)?, "{}", json!(data))?;
        Ok(imgur_url)
    }

    async fn upload(image_url: &str, client_id: &str) -> Result<String, ImgurError> {
        let img = reqwest::get(image_url).await?.bytes().await?;
        let client = reqwest::Client::new();
        let response = client
            .post("https://api.imgur.com/3/image")
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Client-ID {}", client_id),
            )
            .body(img)
            .send()
            .await?;
        let val: Value = serde_json::from_str(&response.text().await?)?;

        Ok(val["data"]["link"]
            .as_str()
            .expect("imgur returned no image url!")
            .to_string())
    }
}
