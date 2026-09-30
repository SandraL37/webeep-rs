use serde::Serialize;
use tauri_plugin_http::reqwest;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Invalid session token. Please re-authenticate.")]
    InvalidToken,

    #[error("Moodle API error: {0}")]
    Moodle(String),

    #[error("Parse error: {0}")]
    Parse(#[from] serde_json::Error),

    #[error("User not authenticated")]
    Unauthenticated,
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
