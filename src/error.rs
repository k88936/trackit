use thiserror::Error;

#[derive(Error, Debug)]
pub enum TrackItError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("API error: {0}")]
    Api(#[from] api::apis::Error<serde_json::Value>),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Missing configuration: {0}")]
    MissingConfig(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("TOML error: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("Dialog error: {0}")]
    Dialog(#[from] dialoguer::Error),
}

pub type Result<T> = std::result::Result<T, TrackItError>;
