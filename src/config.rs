use serde::Deserialize;
use std::path::Path;

#[derive(Debug)]
pub enum ConfigError {
    FileNotFound,
    NotFile,
    Io(std::io::Error),
    TomlDe(toml::de::Error),
}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        ConfigError::Io(e)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(e: toml::de::Error) -> Self {
        ConfigError::TomlDe(e)
    }
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub token: String,
}

pub fn load_config(p: &Path) -> Result<Config, ConfigError> {
    if !p.exists() {
        return Err(ConfigError::FileNotFound);
    }

    if !p.is_file() {
        return Err(ConfigError::NotFile);
    }

    let data = std::fs::read(p)?;
    let config: Config = toml::from_slice(&data)?;

    //TODO: Validate token
    Ok(config)
}
