use crate::error::{Result, TrackItError};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub url: Option<String>,
    pub token: Option<String>,
    pub default_project: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            url: None,
            token: None,
            default_project: None,
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

    fn get_save_path() -> Result<PathBuf> {
        if let Some(home) = std::env::var_os("HOME") {
            return Ok(PathBuf::from(home).join("trackit.toml"));
        }

        if let Some(profile) = std::env::var_os("USERPROFILE") {
            return Ok(PathBuf::from(profile).join("trackit.toml"));
        }

        Ok(PathBuf::from("trackit.toml"))
    }

    pub fn config_file() -> Result<PathBuf> {
        let paths = Self::get_config_search_paths();

        for path in &paths {
            if path.exists() {
                return Ok(path.clone());
            }
        }

        Self::get_save_path()
    }

    pub fn config_search_paths() -> Vec<PathBuf> {
        Self::get_config_search_paths()
    }

    pub fn load() -> Result<Self> {
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

    pub fn save(&self) -> Result<()> {
        let path = Self::get_save_path()?;
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn get_url(&self) -> Result<String> {
        self.url.clone()
            .or_else(|| std::env::var("YOUTRACK_URL").ok())
            .ok_or_else(|| TrackItError::MissingConfig("YouTrack URL not configured. Set YOUTRACK_URL or run 'trackit config set url <URL>'".to_string()))
    }

    pub fn get_token(&self) -> Result<String> {
        self.token.clone()
            .or_else(|| std::env::var("YOUTRACK_TOKEN").ok())
            .ok_or_else(|| TrackItError::MissingConfig("API token not configured. Set YOUTRACK_TOKEN or run 'trackit config set token <TOKEN>'".to_string()))
    }

    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "url" => self.url = Some(value.to_string()),
            "token" => self.token = Some(value.to_string()),
            "default-project" => self.default_project = Some(value.to_string()),
            _ => return Err(TrackItError::Config(format!("Unknown config key: {}", key))),
        }
        Ok(())
    }
}
