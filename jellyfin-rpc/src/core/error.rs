use std::env;

/// Error type for the config module.
#[derive(Debug)]
pub enum ConfigError {
    /// Returns when it can't find the config file.
    MissingConfig(String),
    /// Returns when it can't read the config file.
    Io(String),
    /// Returns when it's unable to parse the config file to the Config struct.
    Json(String),
    /// Returns when environment variables fail to be read.
    VarError(String),
}

impl From<&'static str> for ConfigError {
    fn from(value: &'static str) -> Self {
        Self::MissingConfig(value.to_string())
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(format!("Unable to open file: {}", value))
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(format!("Unable to parse config: {}", value))
    }
}

impl From<env::VarError> for ConfigError {
    fn from(value: env::VarError) -> Self {
        Self::VarError(format!("Unable to get environment variables: {}", value))
    }
}

/// Error type for the imgur module.
#[derive(Debug)]
pub enum ImgurError {
    /// Returns when the response from the Imgur API is invalid.
    ///
    /// This is usually due to a bad API key or something wrong with the image its trying to upload.
    InvalidResponse,
    /// Returns on errors in the reqwest library, can happen when trying to upload a file.
    Reqwest(String),
    /// Returns when it can't read the urls.json file.
    Io(String),
    /// Returns when it can't parse the urls.json file.
    Json(String),
    /// Returns when environment variables fail to be read.
    VarError(String),
    /// Returns when a required `Option<T>` is `None`.
    None,
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

/// Error type for the jellyfin module
// TODO: Rename to `JellyfinError`
#[derive(Debug)]
pub enum ContentError {
    /// Returns on errors in the reqwest library, can happen when trying to access the Jellyfin server.
    Reqwest(reqwest::Error, String),
    /// Returns when the reply from jellyfin can't be parsed to the needed types.
    Json(serde_json::Error),
}

impl From<serde_json::Error> for ContentError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<reqwest::Error> for ContentError {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(value, "Is your Jellyfin URL set correctly?".to_string())
    }
}
