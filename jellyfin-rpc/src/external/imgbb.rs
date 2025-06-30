use std::{
    fs::{self, File, OpenOptions},
    io::{Error, ErrorKind, Write},
    path::Path,
};

use base64::{Engine as _, engine::general_purpose};
use log::debug;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{Client, JfResult};

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

#[derive(Deserialize)]
struct ImgbbResponse {
    data: ImgbbData,
    success: bool,
}

#[derive(Deserialize)]
struct ImgbbData {
    id: String,
    url: String,
    display_url: String,
}

pub fn get_image(client: &Client) -> JfResult<Url> {
    debug!("ImgBB get_image called");
    let mut image_urls = read_file(client)?;
    debug!("Loaded {} cached ImgBB URLs", image_urls.len());

    if let Some(image_url) = image_urls
        .iter()
        .find(|image_url| client.session.as_ref().unwrap().item_id == image_url.id)
    {
        debug!("Found cached ImgBB URL for item {}", image_url.id);
        Ok(Url::parse(&image_url.url)?)
    } else {
        debug!("No cached URL found, uploading to ImgBB");
        let imgbb_url = upload(client)?;

        let image_url = ImageUrl::new(
            &client.session.as_ref().unwrap().item_id,
            imgbb_url.as_str(),
        );

        image_urls.push(image_url);

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&client.imgbb_options.urls_location)?;

        file.write_all(serde_json::to_string(&image_urls)?.as_bytes())?;

        let _ = file.flush();

        debug!("Cached new ImgBB URL: {}", imgbb_url);
        Ok(imgbb_url)
    }
}

fn read_file(client: &Client) -> JfResult<Vec<ImageUrl>> {
    if let Ok(contents_raw) = fs::read_to_string(&client.imgbb_options.urls_location) {
        if let Ok(contents) = serde_json::from_str::<Vec<ImageUrl>>(&contents_raw) {
            return Ok(contents);
        }
    }

    let path = Path::new(&client.imgbb_options.urls_location)
        .parent()
        .ok_or(Error::new(
            ErrorKind::Other,
            "Can't find parent folder of urls.json",
        ))?;

    fs::create_dir_all(path)?;

    let mut file = File::create(client.imgbb_options.urls_location.clone())?;

    let new: Vec<ImageUrl> = vec![];

    file.write_all(serde_json::to_string(&new)?.as_bytes())?;

    let _ = file.flush();

    Ok(new)
}

fn upload(client: &Client) -> JfResult<Url> {
    debug!("Starting ImgBB upload process");
    
    if client.imgbb_options.api_key.is_empty() {
        return Err("ImgBB API key is empty".into());
    }
    
    let image_bytes = client.reqwest.get(client.get_image()?).send()?.bytes()?;
    debug!("Downloaded image bytes: {} bytes", image_bytes.len());

    // Encode image as base64
    let base64_image = general_purpose::STANDARD.encode(&image_bytes);
    debug!("Encoded image to base64, length: {} characters", base64_image.len());

    let imgbb_client = reqwest::blocking::Client::builder().build()?;

    let form = reqwest::blocking::multipart::Form::new()
        .text("image", base64_image);

    let api_url = format!("https://api.imgbb.com/1/upload?key={}", client.imgbb_options.api_key);
    debug!("Uploading to ImgBB API: {}", api_url.replace(&client.imgbb_options.api_key, "***"));

    let response = imgbb_client
        .post(&api_url)
        .multipart(form)
        .send()?;

    debug!("ImgBB response status: {}", response.status());
    
    if !response.status().is_success() {
        let error_text = response.text()?;
        debug!("ImgBB API error response: {}", error_text);
        return Err(format!("ImgBB API error: {}", error_text).into());
    }

    let response_text = response.text()?;
    debug!("ImgBB response body: {}", response_text);

    let res: ImgbbResponse = serde_json::from_str(&response_text)?;
    
    if !res.success {
        return Err("ImgBB upload failed".into());
    }

    debug!("ImgBB upload successful, URL: {}", res.data.url);
    Ok(Url::parse(res.data.url.as_str())?)
}