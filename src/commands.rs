pub mod announce;
pub mod invite;
pub mod join;
pub mod leave;
pub mod movie_quote;
pub mod ping;
pub mod play;
pub mod reddit;
pub mod stop;
pub mod vaporwave;
pub mod zalgo;

pub use crate::commands::{
    announce::ANNOUNCE_COMMAND,
    invite::INVITE_COMMAND,
    join::JOIN_COMMAND,
    leave::LEAVE_COMMAND,
    movie_quote::MOVIE_QUOTE_COMMAND,
    ping::PING_COMMAND,
    play::PLAY_COMMAND,
    reddit::REDDIT_COMMAND,
    stop::STOP_COMMAND,
    vaporwave::VAPORWAVE_COMMAND,
    zalgo::ZALGO_COMMAND,
};
