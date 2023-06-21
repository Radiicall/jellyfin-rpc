use colored::Colorize;

use crate::VERSION;
pub async fn checker() {
    let current = VERSION.unwrap_or("0.0.0").to_string();
    let latest = get_latest_github()
        .await
        .unwrap_or(
            current.clone()
        );
    if latest != current {
            eprintln!("{} (Current: v{}, Latest: v{})\n{}\n{}\n{}",
            "You are not running the latest version of Jellyfin-RPC".red().bold(),
            current,
            latest,
            "A newer version can be found at".red().bold(),
            "https://github.com/Radiicall/jellyfin-rpc/releases/latest".green().bold(),
            "This can be safely ignored if you are running a prerelease version".bold());
            std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

async fn get_latest_github() -> Result<String, reqwest::Error> {
    let url = reqwest::get("https://github.com/Radiicall/jellyfin-rpc/releases/latest")
        .await?
        .url()
        .as_str()
        .trim_start_matches("https://github.com/Radiicall/jellyfin-rpc/releases/tag/")
        .to_string();
    Ok(url)
}
