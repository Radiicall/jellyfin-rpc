use serde_json::Value;
use std::io::Write;

/*
    TODO: Comments
*/

pub async fn get_image_imgur(
    image_url: &String,
    item_id: &String,
    client_id: &String,
    image_urls_file: Option<String>,
) -> Result<String, reqwest::Error> {
    let file = image_urls_file.unwrap_or_else(|| {
        if cfg!(not(windows)) {
            if std::env::var("USER").unwrap() != *"root" {
                std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
                    let mut dir = std::env::var("HOME").unwrap();
                    dir.push_str("/.config/jellyfin-rpc/urls.json");
                    dir
                })
            } else {
                "/etc/jellyfin-rpc/urls.json".to_string()
            }
        } else {
            let mut dir = std::env::var("APPDATA").unwrap();
            dir.push_str(r"\jellyfin-rpc\urls.json");
            dir
        }
    });
    let mut json = read_file(file.clone());
    if json.get(item_id).is_some() {
        return Ok(json[item_id].as_str().unwrap().to_string());
    }

    Ok(write_file(file, image_url, item_id, client_id, &mut json).await)
}

fn read_file(file: String) -> Value {
    let content = std::fs::read_to_string(file.clone())
        .ok()
        .unwrap_or_else(|| {
            std::fs::create_dir_all(std::path::Path::new(&file).parent().unwrap()).ok();
            std::fs::File::create(file.clone()).ok().map(|mut file| {
                write!(file, "{{\n}}").ok();
                file
            });
            std::fs::read_to_string(file).unwrap()
        });
    let json: Value = serde_json::from_str(&content).unwrap();
    json
}

async fn write_file(
    file: String,
    image_url: &String,
    item_id: &String,
    client_id: &String,
    json: &mut Value,
) -> String {
    let mut new_data = serde_json::Map::new();
    let imgur_url = upload_image(image_url, client_id).await.unwrap();
    new_data.insert(item_id.to_string(), serde_json::json!(imgur_url));

    let data = json.as_object_mut().unwrap();
    data.append(&mut new_data);

    write!(
        std::fs::File::create(file).unwrap(),
        "{}",
        serde_json::json!(data)
    )
    .unwrap();
    imgur_url
}

async fn upload_image(image_url: &String, client_id: &String) -> Result<String, reqwest::Error> {
    macro_rules! imgur_api (
        ($url: expr) => (
            concat!("https://api.imgur.com/3/", $url)
        );
    );

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
    let val: Value = serde_json::from_str(&response.text().await?).unwrap();

    Ok(val["data"]["link"].as_str().unwrap().to_string())
}
