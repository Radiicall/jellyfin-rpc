use std::{error::Error, fmt::Display};

/// Error type
#[derive(Debug)]
pub enum JfError {
    /// MediaType returned from jellyfin is of type None,
    /// this should be reported on github
    UnrecognizedMediaType,
    /// Content is in blacklist
    ContentBlacklist,
}

impl Error for JfError {}

impl Display for JfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JfError::UnrecognizedMediaType => write!(f, "unrecognized media type"),
            JfError::ContentBlacklist => write!(f, "content is blacklisted"),
        }
    }
}
