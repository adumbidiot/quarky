use anyhow::Context;
use serde::Deserialize;
use std::path::Path;

/// The bot config
#[derive(Deserialize, Debug)]
pub struct Config {
    /// The bot prefix
    pub prefix: String,

    /// The bot token
    pub token: String,

    /// The twitter config
    pub twitter: TwitterConfig,
}

impl Config {
    /// Load the config from a path
    pub fn load<P>(path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        let data = std::fs::read_to_string(path).context("failed to read config")?;
        let config = toml::from_str(&data).context("failed to parse config")?;

        // TODO: Validate token
        Ok(config)
    }
}

/// The twitter config
#[derive(Deserialize, Debug)]
pub struct TwitterConfig {
    /// The twitter token
    pub bearer_token: String,
}
