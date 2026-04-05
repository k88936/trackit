use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error;
use crate::error::TrackItError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server_url: Option<String>,
    pub token: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_url: None,
            token: None,
        }
    }
}

impl Config {
    fn get_config_search_paths() -> Vec<PathBuf> {
        let mut paths = vec![];

        paths.push(PathBuf::from("trackit.toml"));

        if let Some(home) = std::env::var_os("HOME") {
            paths.push(PathBuf::from(home).join("trackit.toml"));
        }

        if let Some(profile) = std::env::var_os("USERPROFILE") {
            paths.push(PathBuf::from(profile).join("trackit.toml"));
        }

        paths
    }

    fn get_save_path() -> error::Result<PathBuf> {
        if let Some(home) = std::env::var_os("HOME") {
            return Ok(PathBuf::from(home).join("trackit.toml"));
        }

        if let Some(profile) = std::env::var_os("USERPROFILE") {
            return Ok(PathBuf::from(profile).join("trackit.toml"));
        }

        Ok(PathBuf::from("trackit.toml"))
    }

    pub fn config_file() -> error::Result<PathBuf> {
        let paths = Self::get_config_search_paths();

        for path in &paths {
            if path.exists() {
                return Ok(path.clone());
            }
        }

        Self::get_save_path()
    }

    pub fn load() -> error::Result<Self> {
        let paths = Self::get_config_search_paths();

        for path in &paths {
            if path.exists() {
                let content = std::fs::read_to_string(path)?;
                let config: Config = toml::from_str(&content)?;
                return Ok(config);
            }
        }

        Ok(Self::default())
    }

    pub fn save(&self) -> error::Result<()> {
        let path = Self::get_save_path()?;
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn resolve_url(&self, cli_url: Option<&str>) -> error::Result<String> {
        cli_url
            .map(str::to_string)
            .or_else(|| self.server_url.clone())
            .or_else(|| std::env::var("YOUTRACK_URL").ok())
            .ok_or_else(|| {
                TrackItError::MissingConfig(
                    "YouTrack URL not configured. Use --url, set trackit.toml, set YOUTRACK_URL, or run 'trackit setup-wizard'".to_string(),
                )
            })
    }

    pub fn resolve_token(&self, cli_token: Option<&str>) -> error::Result<String> {
        cli_token
            .map(str::to_string)
            .or_else(|| self.token.clone())
            .or_else(|| std::env::var("YOUTRACK_TOKEN").ok())
            .ok_or_else(|| {
                TrackItError::MissingConfig(
                    "API token not configured. Use --token, set trackit.toml, set YOUTRACK_TOKEN, or run 'trackit setup-wizard'".to_string(),
                )
            })
    }
}
