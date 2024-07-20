use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum JfError {
    UnrecognizedMediaType,
    ContentBlacklist,
}

impl Error for JfError {}

impl Display for JfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JfError::UnrecognizedMediaType => write!(f, "Unrecognized media type"),
            JfError::ContentBlacklist => write!(f, "Content is blacklisted"),
        }
    }
}
