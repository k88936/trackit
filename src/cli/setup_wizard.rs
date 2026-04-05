use crate::config::Config;
use crate::error::Result;
use dialoguer::{Input, Password};

pub async fn run_setup_wizard() -> Result<()> {
    let mut config = Config::load().unwrap_or_default();

    let url: String = Input::new()
        .with_prompt("YouTrack URL")
        .default(config.server_url.clone().unwrap_or_default())
        .interact_text()?;

    let token: String = Password::new().with_prompt("API Token").interact()?;

    config.server_url = Some(url);
    config.token = Some(token);

    config.save()?;
    println!("Configuration saved to {:?}", Config::config_file()?);

    Ok(())
}
