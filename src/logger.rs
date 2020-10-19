use fern::colors::{
    Color,
    ColoredLevelConfig,
};

#[derive(Debug, thiserror::Error)]
pub enum LoggerError {
    #[error(transparent)]
    SetLogger(#[from] log::SetLoggerError),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub fn setup() -> Result<(), LoggerError> {
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
        .chain(fern::log_file("quarky.log")?)
        .apply()?;
    Ok(())
}
