use api::apis::configuration::Configuration;

use crate::error::Result;

use super::utils::normalize_base_path;

pub struct YouTrackClient {
    pub(super) configuration: Configuration,
}

impl YouTrackClient {
    pub fn new(base_url: &str, token: &str) -> Result<Self> {
        let mut configuration = Configuration::new();
        configuration.base_path = normalize_base_path(base_url)?;
        configuration.bearer_access_token = Some(token.to_string());
        configuration.user_agent = Some(format!("trackit/{}", env!("CARGO_PKG_VERSION")));

        Ok(Self { configuration })
    }
}
