use reqwest::{Response};
use serde_json::Value;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use colored::Colorize;

const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::from_path(std::env::current_exe()?.parent().unwrap().join(".env")).ok();
    let rpc_client_id = dotenv::var("DISCORD_APPLICATION_ID").unwrap_or_else(|_| "1053747938519679018".to_string());
    let url = dotenv::var("JELLYFIN_URL").unwrap_or_else(|_| "".to_string());
    let api_key = dotenv::var("JELLYFIN_API_KEY").unwrap_or_else(|_| "".to_string());
    let username = dotenv::var("JELLYFIN_USERNAME").unwrap_or_else(|_| "".to_string());

    checkargs();
    
    
    println!("{}\n                          {}", "//////////////////////////////////////////////////////////////////".bold(), "Jellyfin-RPC".bright_blue());

    if rpc_client_id.is_empty() || url.is_empty() || api_key.is_empty() || username.is_empty() {
        panic!("Please make a file called .env and populate it with the needed variables (https://github.com/Radiicall/jellyfin-rpc#setup)");
    }

    let mut connected: bool = false;
    let mut drpc = DiscordIpcClient::new(rpc_client_id.as_str()).expect("Failed to create Discord RPC client, discord is down or the Client ID is invalid.");
    // Start loop
    loop {
        let jfresult = match get_jellyfin_playing(&url, &api_key, &username).await {
            Ok(res) => res,
            Err(_) => vec!["".to_string()],
        };
        let media_type = &jfresult[0];
        if !media_type.is_empty() {
            let mut extname: Vec<&str> = std::vec::Vec::new();
            jfresult[1].split(',').for_each(|p| extname.push(p));
            let mut exturl: Vec<&str> = std::vec::Vec::new();
            jfresult[2].split(',').for_each(|p| exturl.push(p));
            let details = "Watching ".to_owned() + jfresult[3].trim_matches('"');
            let endtime = jfresult[4].parse::<i64>().unwrap();
            let state_message = "".to_owned() + &jfresult[5];

            if !connected {
                // Start up the client connection, so that we can actually send and receive stuff
                connect(&mut drpc);
                println!("{}\n{}\n{}\n{}\n{}", "//////////////////////////////////////////////////////////////////".bold(), "Connected to Discord RPC client".bright_green().bold(), "//////////////////////////////////////////////////////////////////".bold(), details.bright_cyan().bold(), state_message.bright_cyan().bold());

                // Set connected to true so that we don't try to connect again
                connected = true;
            }

            // Set the activity
            let mut rpcbuttons: Vec<activity::Button> = std::vec::Vec::new();
            for i in 0..extname.len() {
                rpcbuttons.push(activity::Button::new(
                    extname[i],
                    exturl[i],
                ));
            }
            
            drpc.set_activity(
                setactivity(&state_message, &details, endtime, rpcbuttons)
            ).expect("Failed to set activity");
            
        } else if connected {
            // Disconnect from the client
            drpc.close().expect("Failed to close Discord RPC client");
            // Set connected to false so that we dont try to disconnect again
            connected = false;
            println!("{}", "Disconnected from Discord RPC client".bright_red().bold());
        }
    // Sleep for 2 seconds
    std::thread::sleep(std::time::Duration::from_secs(2));
    }
}

async fn get_jellyfin_playing(url: &String, api_key: &String, username: &String) -> Result<Vec<String>, reqwest::Error> {
    /*
    Make a request to the jellyfin server with the api key, wait for response, then get body text and convert to json.
    Make a for loop for the json, jellyfin makes a list of all active sessions so we need to look through them.
    Check if the username matches the one in the config, if we dont do this then it would literally grab any user and set it as status.
    It then checks if anything is actually playing on said user's session.
    From here it runs all of the other functions to get the proper information, then returns it to the main loop.
    */
    let url = format!("{}/Sessions?api_key={}", url.trim_end_matches("/"), api_key);

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

                    return Ok(get_currently_watching(nowplayingitem, &extsrv[0], &extsrv[1], timeleft))
                },
            };
        }
    }
    Ok(vec!["".to_owned()])
}

fn get_external_services(npi: &Value) -> Vec<String> {
    /*
    The is for external services that might host info about what we're currently watching.
    It first checks if they actually exist by checking if the first thing in the array is an object.
    If it is then it creates a for loop and pushes every "name" and "url" to 2 strings with commas seperating.
    When the for loop reaches 2 it breaks (this is the max number of buttons in discord rich presence),
    then it removes the trailing commas from the strings.
    */
    let mut extname: String = "".to_string();
    let mut exturl: String = "".to_string();
    match npi.get("ExternalUrls") {
        None => (),
        extsrv => {
            if extsrv.expect("Couldn't find ExternalUrls")[0].is_object() {
                let mut x = 0;
                for i in extsrv.expect("Couldn't find ExternalUrls").as_array().unwrap() {
                    extname.push_str(i.get("Name").unwrap().as_str().unwrap());
                    exturl.push_str(i.get("Url").unwrap().as_str().unwrap());
                    extname.push(',');
                    exturl.push(',');
                    x += 1;
                    if x == 2 {
                        break
                    }
                }
                extname = extname.trim_end_matches(',').to_string();
                exturl = exturl.trim_end_matches(',').to_string();
                return vec![extname, exturl]
            }
        }
    };
    vec!["".to_string()]
}

fn get_end_timer(npi: &Value, json: &Value) -> String {
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
        .as_i64().unwrap_or(0);
    position_ticks /= 10000000;
    let mut runtime_ticks = npi
    .get("RunTimeTicks").unwrap_or(&serde_json::json!(0))
    .as_i64().unwrap_or(0);
    runtime_ticks /= 10000000;
    (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64 + (runtime_ticks - position_ticks)).to_string()
}

fn get_currently_watching(npi: &Value, extname: &String, exturl: &String, timeleft: String) -> Vec<String> {
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
    let name = npi.get("Name").expect("Couldn't find Name").to_string();
    if npi.get("Type").unwrap().as_str().unwrap() == "Episode" {
        let itemtype = "episode".to_owned();
        let series_name = npi.get("SeriesName").expect("Couldn't find SeriesName.").to_string();
        let season = npi.get("ParentIndexNumber").expect("Couldn't find ParentIndexNumber.").to_string();
        let episode = npi.get("IndexNumber").expect("Couldn't find IndexNumber.").to_string();

        let msg = "S".to_owned() + &season + "E" + &episode + " " + &name[1..name.len() - 1];
        vec![itemtype, extname.to_owned(), exturl.to_owned(), series_name, timeleft, msg]

    } else if npi.get("Type").unwrap().as_str().unwrap() == "Movie" {
        let itemtype = "movie".to_owned();
        let mut episode = "".to_string();
        match npi.get("Genres") {
            None => (),
            genres => {
                for i in genres.unwrap().as_array().unwrap() {
                    episode.push_str(i.as_str().unwrap());
                    episode.push_str(", ");
                }
                episode = episode[0..episode.len() - 2].to_string();
            }
        };

        return vec![itemtype, extname.to_owned(), exturl.to_owned(), name, timeleft, episode];
    } else {
        return vec!["".to_string()]
    }
}

fn setactivity<'a>(state_message: &'a String, details: &'a str, endtime: i64, rpcbuttons: Vec<activity::Button<'a>>) -> activity::Activity<'a> {
    let mut payload = activity::Activity::new()
        .details(details)
        .assets(
            activity::Assets::new()
                .large_image("https://s1.qwant.com/thumbr/0x380/0/6/aec9d939d464cc4e3b4c9d7879936fbc61901ccd9847d45c68a3ce2dbd86f0/cover.jpg?u=https%3A%2F%2Farchive.org%2Fdownload%2Fgithub.com-jellyfin-jellyfin_-_2020-09-15_17-17-00%2Fcover.jpg")
                .large_text("https://github.com/Radiicall/jellyfin-rpc") 
        )
        .timestamps(activity::Timestamps::new()
            .end(endtime)
        );

    if !state_message.is_empty() {
        payload = payload.clone().state(state_message);
    }
    if !rpcbuttons.is_empty() {
        payload = payload.clone().buttons(rpcbuttons);
    }
    payload
}

fn connect(drpc: &mut DiscordIpcClient) {
    loop {
        match drpc.connect() {
            Ok(result) => result,
            Err(_) => {
                println!("{}", "Failed to connect, retrying in 10 seconds".red().bold()); 
                std::thread::sleep(std::time::Duration::from_secs(10)); 
                continue
            },
        };
        break;
    }
}

fn checkargs() {
    match std::env::args().nth(1).unwrap_or_default().as_str() {
        "-h" | "--help" => {
            println!(
r#"Start rich presence for Jellyfin

Usage: {} [Options]

Options:
    -h, --help     shows this page
    -v, --version  shows version information
"#,
            std::env::args().next().unwrap());
            std::process::exit(0)
        },
        "-v" | "--version" => {
            println!("jellyfin-rpc v{}", VERSION.unwrap_or("unknown"));
            std::process::exit(0)
        }
        _ => (),
    }
}
