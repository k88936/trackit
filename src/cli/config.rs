use clap::Subcommand;
use dialoguer::{Input, Password};
use crate::config::Config;
use crate::error::Result;

#[derive(Subcommand)]
pub enum ConfigCommand {
    #[command(about = "Interactive setup wizard")]
    Init,

    #[command(about = "Display current configuration")]
    Show,

    #[command(about = "Set config value")]
    Set {
        #[arg(help = "Config key (url, token, default-project)")]
        key: String,
        
        #[arg(help = "Config value")]
        value: String,
    },

    #[command(about = "Show config file location")]
    Path,
}

pub async fn handle_config_command(cmd: ConfigCommand) -> Result<()> {
    match cmd {
        ConfigCommand::Init => {
            let mut config = Config::load().unwrap_or_default();
            
            let url: String = Input::new()
                .with_prompt("YouTrack URL")
                .default(config.url.clone().unwrap_or_default())
                .interact_text()?;
            
            let token: String = Password::new()
                .with_prompt("API Token")
                .interact()?;
            
            let default_project: String = Input::new()
                .with_prompt("Default project (optional)")
                .default(config.default_project.clone().unwrap_or_default())
                .allow_empty(true)
                .interact_text()?;

            config.url = Some(url);
            config.token = Some(token);
            if !default_project.is_empty() {
                config.default_project = Some(default_project);
            }

            config.save()?;
            println!("Configuration saved to {:?}", Config::config_file()?);
        }
        
        ConfigCommand::Show => {
            let config = Config::load()?;
            println!("URL: {}", config.url.as_deref().unwrap_or("not set"));
            println!("Token: {}", if config.token.is_some() { "******" } else { "not set" });
            println!("Default project: {}", config.default_project.as_deref().unwrap_or("not set"));
        }
        
        ConfigCommand::Set { key, value } => {
            let mut config = Config::load().unwrap_or_default();
            config.set(&key, &value)?;
            config.save()?;
            println!("Set {} to: {}", key, value);
        }
        
        ConfigCommand::Path => {
            let paths = Config::config_search_paths();
            println!("Config search paths (in order):");
            for path in paths {
                let exists = if path.exists() { " (exists)" } else { "" };
                println!("  {:?}{}", path, exists);
            }
        }
    }
    Ok(())
}
