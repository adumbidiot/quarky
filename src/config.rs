use anyhow::Context;
use camino::{
    Utf8Path,
    Utf8PathBuf,
};
use serde::Deserialize;

/// The bot config
#[derive(Deserialize, Debug)]
pub struct Config {
    /// The bot prefix
    pub prefix: String,

    /// The bot token
    pub token: String,

    /// The twitter config
    pub twitter: TwitterConfig,

    /// The log config
    #[serde(default)]
    pub log: LogConfig,
}

impl Config {
    /// Load the config from a path
    pub fn load<P>(path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Utf8Path>,
    {
        let data = std::fs::read_to_string(path.as_ref()).context("failed to read config")?;
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

/// The log config
#[derive(Deserialize, Debug)]
pub struct LogConfig {
    /// The log file location
    #[serde(rename = "log-file", default = "LogConfig::default_log_file")]
    pub log_file: Utf8PathBuf,
}

impl LogConfig {
    /// Get the default value of the `log_file` field
    fn default_log_file() -> Utf8PathBuf {
        Utf8PathBuf::from("quarky.log")
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            log_file: Self::default_log_file(),
        }
    }
}
