use std::env;

#[derive(Debug)]
pub enum ConfigError {
    MissingConfig(String),
    Io(String),
    Json(String),
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

#[derive(Debug)]
pub enum ImgurError {
    InvalidResponse,
    Reqwest(String),
    Io(String),
    Json(String),
    VarError(String),
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

#[derive(Debug)]
pub enum ContentError {
    Reqwest(reqwest::Error, String),
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
