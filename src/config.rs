use serde::Deserialize;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    TomlDe(#[from] toml::de::Error),
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub token: String,
    pub twitter: TwitterConfig,
}

#[derive(Deserialize, Debug)]
pub struct TwitterConfig {
    pub bearer_token: String,
}

pub fn load_config(p: &Path) -> Result<Config, ConfigError> {
    let data = std::fs::read(p)?;
    let config: Config = toml::from_slice(&data)?;

    // TODO: Validate token
    Ok(config)
}
