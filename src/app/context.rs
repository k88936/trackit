use crate::config::Config;
use crate::error::Result;
use crate::youtrack::YouTrackClient;

#[derive(Clone)]
pub struct GlobalOpts {
    pub json: bool,
    pub url: Option<String>,
    pub token: Option<String>,
}

pub fn build_client(global: &GlobalOpts) -> Result<YouTrackClient> {
    let config = Config::load()?;
    let url = config.resolve_url(global.url.as_deref())?;
    let token = config.resolve_token(global.token.as_deref())?;
    YouTrackClient::new(&url, &token)
}
