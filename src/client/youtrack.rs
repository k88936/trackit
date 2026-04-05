use crate::config::Config;
use crate::error::{Result, TrackItError};
use api::apis::configuration::Configuration;

pub struct YouTrackClient {
    pub config: Configuration,
}

impl YouTrackClient {
    pub fn from_config(config: &Config) -> Result<Self> {
        let url = config.get_url()?;
        let token = config.get_token()?;

        let base_path = if url.ends_with("/api") {
            url
        } else {
            format!("{}/api", url.trim_end_matches('/'))
        };

        let mut api_config = Configuration::new();
        api_config.base_path = base_path;
        api_config.bearer_access_token = Some(token);

        Ok(Self { config: api_config })
    }

    pub fn with_overrides(
        config: &Config,
        url_override: Option<String>,
        token_override: Option<String>,
    ) -> Result<Self> {
        let url = url_override
            .or_else(|| config.url.clone())
            .or_else(|| std::env::var("YOUTRACK_URL").ok())
            .ok_or_else(|| {
                TrackItError::MissingConfig("YouTrack URL not configured".to_string())
            })?;

        let token = token_override
            .or_else(|| config.token.clone())
            .or_else(|| std::env::var("YOUTRACK_TOKEN").ok())
            .ok_or_else(|| TrackItError::MissingConfig("API token not configured".to_string()))?;

        let base_path = if url.ends_with("/api") {
            url
        } else {
            format!("{}/api", url.trim_end_matches('/'))
        };

        let mut api_config = Configuration::new();
        api_config.base_path = base_path;
        api_config.bearer_access_token = Some(token);

        Ok(Self { config: api_config })
    }
}
