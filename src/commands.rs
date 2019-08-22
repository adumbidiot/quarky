pub mod announce;
pub mod movie_quote;
pub mod ping;
pub mod reddit;
pub mod zalgo;

pub use crate::commands::{
    announce::ANNOUNCE_COMMAND,
    movie_quote::MOVIE_QUOTE_COMMAND,
    ping::PING_COMMAND,
    reddit::REDDIT_COMMAND,
    zalgo::ZALGO_COMMAND,
};
