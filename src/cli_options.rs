use camino::Utf8PathBuf;

#[derive(Debug, argh::FromArgs)]
#[argh(description = "The Quarky discord bot")]
pub struct CliOptions {
    /// The path to the config
    #[argh(
        option,
        short = 'c',
        long = "--config",
        description = "the path to the config file",
        default = "Utf8PathBuf::from(\"./config.toml\")"
    )]
    pub config: Utf8PathBuf,
}
