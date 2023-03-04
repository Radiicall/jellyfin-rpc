use serde_json::Value;

pub struct Content {
    pub media_type: String,
    pub details: String,
    pub state_message: String,
    pub endtime: Option<i64>,
    pub image_url: String,
    pub external_service_names: Vec<String>,
    pub external_service_urls: Vec<String>,
}

pub async fn get_jellyfin_playing(url: &str, api_key: &String, username: &String, enable_images: &bool) -> Result<Content, reqwest::Error> {
    let sessions: Vec<Value> = serde_json::from_str(
        &reqwest::get(
            format!(
                "{}/Sessions?api_key={}",
                url.trim_end_matches('/'),
                api_key
            )
        ).await?.text().await?)
        .unwrap_or_else(|_|
        panic!("Can't unwrap URL, check if JELLYFIN_URL is correct. Current URL: {}",
            url)
        );
    for session in sessions {
        if Option::is_none(&session.get("UserName")) {
            continue 
        }
        if session["UserName"].as_str().unwrap() != username {
            continue
        }
        if Option::is_none(&session.get("NowPlayingItem")) {
            continue
        }

        let now_playing_item = &session["NowPlayingItem"];

        let external_services = get_external_services(now_playing_item).await;

        let main = get_currently_watching(now_playing_item).await;

        let mut image_url: String = "".to_string();
        if enable_images == &true {
            image_url = get_image_jf(url, main[3].clone()).await;
        }

        return Ok(Content {
            media_type: main[0].clone(),
            details: main[1].clone(),
            state_message: main[2].clone(),
            endtime: get_end_timer(now_playing_item, &session).await,
            image_url,
            external_service_names: external_services[0].clone(),
            external_service_urls: external_services[1].clone(),
        })
    }
    Ok(Content {
        media_type: "".to_string(),
        details: "".to_string(),
        state_message: "".to_string(),
        endtime: Some(0),
        image_url: "".to_string(),
        external_service_names: vec!["".to_string()],
        external_service_urls: vec!["".to_string()],
    })
}

async fn get_external_services(now_playing_item: &Value) -> Vec<Vec<String>> {
    let mut external_service_names: Vec<String> = vec![];
    let mut external_service_urls: Vec<String> = vec![];

    let external_services = &now_playing_item["ExternalUrls"];

    if external_services[0].is_object() {
        let mut x = 0;
        for i in external_services.as_array().unwrap() {
            external_service_names.push(i["Name"].as_str().unwrap().to_string());
            external_service_urls.push(i["Url"].as_str().unwrap().to_string());
            x += 1;
            if x == 2 {
                break
            }
        }
    }
    vec![external_service_names, external_service_urls]
}

async fn get_end_timer(now_playing_item: &Value, session: &Value) -> Option<i64> {
    if !session["PlayState"]["IsPaused"].as_bool().unwrap() {
        let ticks_to_seconds = 10000000;

        let mut position_ticks = session["PlayState"]["PositionTicks"].as_i64().unwrap_or(0);
        position_ticks /= ticks_to_seconds;
    
        let mut runtime_ticks = now_playing_item["RunTimeTicks"].as_i64().unwrap_or(0);
        runtime_ticks /= ticks_to_seconds;
    
        Some(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64 + (runtime_ticks - position_ticks))
    } else {
        None
    }
}

async fn get_currently_watching(now_playing_item: &Value) -> Vec<String> {
    /*
    This is where we actually get the info for the Movie/Series that we're currently watching.
    First we set the name variable because that's not gonna change either way.
    Then we check if its an "Episode" or a "Movie".
    If its an "Episode" then we set the item type to "episode", get the name of the series, the season and the actual episode number.
    Then we send that off as a Vec<String> along with the external urls and end timer to the main loop.
    If its a "Movie" then we try to fetch the "Genres" with a simple for loop!
    After the for loop is complete we remove the trailing ", " because it looks bad in the presence.
    Then we send it off as a Vec<String> with the external urls and the end timer to the main loop.
    */
    let name = now_playing_item["Name"].as_str().unwrap();
    let item_type: String;
    let item_id: String;
    if now_playing_item["Type"].as_str().unwrap() == "Episode" {
        item_type = "episode".to_owned();
        let series_name = now_playing_item["SeriesName"].as_str().unwrap().to_string();
        item_id = now_playing_item["SeriesId"].as_str().unwrap().to_string();

        let season = now_playing_item["ParentIndexNumber"].to_string();
        let first_episode_number = now_playing_item["IndexNumber"].to_string();
        let mut msg = "S".to_owned() + &season + "E" + &first_episode_number;

        if !Option::is_none(&now_playing_item.get("IndexNumberEnd")) {
            msg += &("-".to_string() + &now_playing_item["IndexNumberEnd"].to_string());
        }

        msg += &(" ".to_string() + name);

        vec![item_type, series_name, msg, item_id]
    } else if now_playing_item["Type"].as_str().unwrap() == "Movie" {
        item_type = "movie".to_owned();
        item_id = now_playing_item["Id"].as_str().unwrap().to_string();
        let mut genres = "".to_string();
        match now_playing_item.get("Genres") {
            None => (),
            genre_array => {
                for i in genre_array.unwrap().as_array().unwrap() {
                    genres.push_str(i.as_str().unwrap());
                    genres.push_str(", ");
                }
                genres = genres[0..genres.len() - 2].to_string();
            }
        };

        vec![item_type, name.to_string(), genres, item_id]
    } else if now_playing_item["Type"].as_str().unwrap() == "Audio" {
        item_type = "music".to_owned();
        item_id = now_playing_item["Id"].as_str().unwrap().to_string();
        let artist: String = now_playing_item["AlbumArtist"].as_str().unwrap().to_string();

        vec![item_type, name.to_string(), artist, item_id]
    } else {
        // Return 4 empty strings to make vector equal length
        vec!["".to_string(), "".to_string(), "".to_string(), "".to_string()]
    }
}

async fn get_image_jf(url: &str, item_id: String) -> String {
    format!(
        "{}/Items/{}/Images/Primary",
        url.trim_end_matches('/'),
        item_id
    )
}
