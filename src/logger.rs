use crate::Config;
use anyhow::Context;
use fern::colors::{
    Color,
    ColoredLevelConfig,
};

/// Setup the logger
pub fn setup(config: &Config) -> anyhow::Result<()> {
    let colors_line = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::Cyan)
        .debug(Color::White)
        .trace(Color::BrightBlack);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                colors_line.color(record.level()),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .level_for("tracing", log::LevelFilter::Warn)
        .level_for("serenity", log::LevelFilter::Warn)
        .level_for(
            "serenity::client::bridge::gateway::shard_runner",
            log::LevelFilter::Error,
        )
        .chain(std::io::stdout())
        .chain(fern::log_file(&config.log.log_file)?)
        .apply()
        .context("failed to set logger")?;
    Ok(())
}
