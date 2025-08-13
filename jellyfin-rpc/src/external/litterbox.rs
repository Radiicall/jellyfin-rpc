use std::{
    fs::{self, File, OpenOptions},
    io::{Error, ErrorKind, Write},
    path::Path, time::SystemTime,
};

use log::{debug};
use serde::{Deserialize, Serialize};
use url::Url;

use reqwest::{blocking::multipart::{ Form, Part }};
use crate::{ Client, JfResult };

#[derive(Deserialize, Serialize)]
struct ImageUrl {
    id: String,
    url: String,
}

impl ImageUrl {
    fn new<T: Into<String>, Y: Into<String>>(id: T, url: Y) -> Self {
        Self {
            id: id.into(),
            url: url.into(),
        }
    }
}

pub fn get_image(client: &Client) -> JfResult<Url> {
    let mut image_urls = read_file(client)?;

    if let Some(image_url) = image_urls
        .iter()
        .find(|image_url| client.session.as_ref().unwrap().item_id == image_url.id)
    {
        debug!("Found image url: {}", image_url.url.clone());

        Ok(Url::parse(&image_url.url)?)
    } else {
        debug!("No cached litterbox image found. Uploading new image");

        let litterbox_url = upload(client)?;

        debug!("Litterbox response: {}", litterbox_url);

        let image_url = ImageUrl::new(
            &client.session.as_ref().unwrap().item_id,
            litterbox_url.as_str(),
        );

        image_urls.push(image_url);

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&client.litterbox_options.urls_location)?;

        file.write_all(serde_json::to_string(&image_urls)?.as_bytes())?;

        let _ = file.flush();

        Ok(litterbox_url)
    }
}

fn read_file(client: &Client) -> JfResult<Vec<ImageUrl>> {
    if let Ok(contents_raw) = fs::read_to_string(&client.litterbox_options.urls_location) {
        if let Ok(contents) = serde_json::from_str::<Vec<ImageUrl>>(&contents_raw) {
            return Ok(contents);
        }
    }

    let path = Path::new(&client.litterbox_options.urls_location)
        .parent()
        .ok_or(Error::new(
            ErrorKind::Other,
            "Can't find parent folder of urls.json",
        ))?;

    fs::create_dir_all(path)?;

    let mut file = File::create(client.litterbox_options.urls_location.clone())?;

    let new: Vec<ImageUrl> = vec![];

    file.write_all(serde_json::to_string(&new)?.as_bytes())?;

    let _ = file.flush();

    Ok(new)
}

fn upload(client: &Client) -> JfResult<Url> {
    let image_bytes = client.reqwest.get(client.get_image()?).send()?.bytes()?;

    debug!("Uploading image to litterbox");

    let litterbox_client = reqwest::blocking::Client::builder().build()?;
    let filename = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .to_string();

    let litterbox_form = Form::new()
        .text("reqtype", "fileupload")
        .text("time", "24h")
        .part("fileToUpload", Part::bytes(image_bytes.to_vec()).file_name(filename));

    let res: String = litterbox_client
        .post("https://litterbox.catbox.moe/resources/internals/api.php")
        .multipart(litterbox_form)
        .send()?
        .text()?;

    debug!("Response from Litterbox: \"{}\"", res.clone());

    Ok(Url::parse(&res)?)
}