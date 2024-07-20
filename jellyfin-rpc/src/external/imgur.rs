use std::{fs::{self, File, OpenOptions}, io::{Error, ErrorKind, Write}, path::Path};

use serde::{Deserialize, Serialize};
use url::Url;

use crate::{Client, JfResult};


#[derive(Deserialize, Serialize)]
struct ImageUrl {
    id: String,
    pub url: String,
}

impl ImageUrl {
    fn new<T: Into<String>, Y: Into<String>>(id: T, url: Y) -> Self {
        Self {
            id: id.into(),
            url: url.into(),
        }
    }
}

#[derive(Deserialize)]
struct ImgurResponse {
    data: Data,
}

#[derive(Deserialize)]
struct Data {
    link: String,
}

pub async fn get_image(client: &Client) -> JfResult<Url> {
    let mut image_urls = read_file(client).await?;


    if let Some(image_url) = image_urls.iter().find(|image_url|  client.session.as_ref().unwrap().item_id == image_url.id) {
        Ok(Url::parse(&image_url.url)?)
    } else {
        let imgur_url = upload(client).await?;

        let image_url = ImageUrl::new(
            &client.session.as_ref().unwrap().item_id,
            imgur_url.as_str()
        );

        image_urls.push(image_url);

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&client.imgur_options.urls_location)?;

        file.write_all(serde_json::to_string(&image_urls)?.as_bytes())?;

        file.flush();
        
        Ok(imgur_url)
    }
}

async fn read_file(client: &Client) -> JfResult<Vec<ImageUrl>> {
    if let Ok(contents_raw) = fs::read_to_string(&client.imgur_options.urls_location) {
        if let Ok(contents) = serde_json::from_str::<Vec<ImageUrl>>(&contents_raw) {
            return Ok(contents)
        }
    }

    let path = Path::new(&client.imgur_options.urls_location).parent()
        .ok_or(Error::new(ErrorKind::Other, "Can't find parent folder of urls.json"))?;

    fs::create_dir_all(path)?;

    let mut file = File::create(client.imgur_options.urls_location.clone())?;

    let new: Vec<ImageUrl> = vec![];

    file.write_all(serde_json::to_string(&new)?.as_bytes())?;

    Ok(new)
}

async fn upload(client: &Client) -> JfResult<Url> {
    let image_bytes = client.reqwest.get(client.get_image().await?)
        .send()
        .await?
        .bytes()
        .await?;

    let res: ImgurResponse = client.reqwest
        .post("https://api.imgur.com/3/image")
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Client-ID {}", client.imgur_options.client_id)
        )
        .body(image_bytes)
        .send()
        .await?
        .json()
        .await?;

    Ok(Url::parse(res.data.link.as_str())?)
}
