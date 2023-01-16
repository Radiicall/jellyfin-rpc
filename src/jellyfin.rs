use reqwest::{Response};
use serde_json::Value;

pub struct Content {
    pub media_type: String,
    pub details: String,
    pub state_message: String,
    pub endtime: i64,
    pub extname: Vec<String>,
    pub exturl: Vec<String>,
}

pub async fn get_jellyfin_playing(url: &str, api_key: &String, username: &String) -> Result<Content, reqwest::Error> {
    /*
    Make a request to the jellyfin server with the api key, wait for response, then get body text and convert to json.
    Make a for loop for the json, jellyfin makes a list of all active sessions so we need to look through them.
    Check if the username matches the one in the config, if we dont do this then it would literally grab any user and set it as status.
    It then checks if anything is actually playing on said user's session.
    From here it runs all of the other functions to get the proper information, then returns it to the main loop.
    */
    let url = format!("{}/Sessions?api_key={}", url.trim_end_matches('/'), api_key);

    let res: Response = reqwest::get(url).await?;
    
    let body = res.text().await?;
    
    let json: Vec<Value> = serde_json::from_str(&body).unwrap_or_else(|_|
        panic!("Can't unwrap URL, check if JELLYFIN_URL is correct. Current URL: {}",
            // Grabbing dotenv var again because i dont know how im supposed to use url variable twice lol
            dotenv::var("JELLYFIN_URL").unwrap_or_else(|_| "".to_string()))
        );
    for i in json {
        if Option::is_none(&i.get("UserName")) {
            continue 
        } else if i.get("UserName").unwrap().as_str().unwrap() == username {
            match i.get("NowPlayingItem") {
                None => continue,
                npi => {
                    // Unwrap the option that was returned
                    let nowplayingitem = npi.unwrap();

                    let extsrv = get_external_services(nowplayingitem);

                    let timeleft = get_end_timer(nowplayingitem, &i);

                    let vector = get_currently_watching(nowplayingitem);
                    return Ok(Content {
                        media_type: vector[0].clone(),
                        details: vector[1].clone(),
                        state_message: vector[2].clone(),
                        endtime: timeleft,
                        extname: extsrv[0].clone(),
                        exturl: extsrv[1].clone(),
                    })

                },
            };
        }
    }
    Ok(Content {
        media_type: "".to_string(),
        details: "".to_string(),
        state_message: "".to_string(),
        endtime: 0,
        extname: vec!["".to_string()],
        exturl: vec!["".to_string()],
    })
}

fn get_external_services(npi: &Value) -> Vec<Vec<String>> {
    /*
    The is for external services that might host info about what we're currently watching.
    It first checks if they actually exist by checking if the first thing in the array is an object.
    If it is then it creates a for loop and pushes every "name" and "url" to 2 strings with commas seperating.
    When the for loop reaches 2 it breaks (this is the max number of buttons in discord rich presence),
    then it removes the trailing commas from the strings.
    */
    let mut extname: Vec<String> = vec![];
    let mut exturl: Vec<String> = vec![];
    match npi.get("ExternalUrls") {
        None => (),
        extsrv => {
            if extsrv.expect("Couldn't find ExternalUrls")[0].is_object() {
                let mut x = 0;
                for i in extsrv.expect("Couldn't find ExternalUrls").as_array().unwrap() {
                    extname.push(i["Name"].as_str().unwrap().to_string());
                    exturl.push(i["Url"].as_str().unwrap().to_string());
                    x += 1;
                    if x == 2 {
                        break
                    }
                }
            }
        }
    };
    vec![extname, exturl]
}

fn get_end_timer(npi: &Value, json: &Value) -> i64 {
    /*
    This is for the end timer,
    it gets the PositionTicks as a string so we can cut off the last 7 digits (millis).
    Then if its empty afterwards we make it 0, then parse it to an i64.
    After that we get the RunTimeTicks, remove the last 7 digits and parse that to an i64.
    PositionTicks is how far into the video we are and RunTimeTicks is how many ticks the video will last for.
    We then do current "SystemTime + (RunTimeTicks - PositionTicks)" and that's how many seconds there are left in the video from the current unix epoch.
    */
    let mut position_ticks = json
        .get("PlayState").unwrap()
        .get("PositionTicks").unwrap_or(&serde_json::json!(0))
        .as_i64().unwrap();
    position_ticks /= 10000000;
    let mut runtime_ticks = npi
    .get("RunTimeTicks").unwrap_or(&serde_json::json!(0))
    .as_i64().unwrap();
    runtime_ticks /= 10000000;
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64 + (runtime_ticks - position_ticks)
}

fn get_currently_watching(npi: &Value) -> Vec<String> {
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
    let name = npi["Name"].as_str().unwrap();
    if npi["Type"].as_str().unwrap() == "Episode" {
        let itemtype = "episode".to_owned();
        let series_name = npi["SeriesName"].as_str().unwrap().to_string();
        let season = npi["ParentIndexNumber"].to_string();
        let episode = npi["IndexNumber"].to_string();

        let msg = "S".to_owned() + &season + "E" + &episode + " " + name;
        vec![itemtype, series_name, msg]

    } else if npi["Type"].as_str().unwrap() == "Movie" {
        let itemtype = "movie".to_owned();
        let mut genre_vector = "".to_string();
        match npi.get("Genres") {
            None => (),
            genres => {
                for i in genres.unwrap().as_array().unwrap() {
                    genre_vector.push_str(i.as_str().unwrap());
                    genre_vector.push_str(", ");
                }
                genre_vector = genre_vector[0..genre_vector.len() - 2].to_string();
            }
        };

        return vec![itemtype, name.to_string(), genre_vector];
    } else {
        // Return 3 empty strings to make vector equal length
        return vec!["".to_string(), "".to_string(), "".to_string()]
    }
}
