mod get_user_tweets;

pub use self::get_user_tweets::{
    GetUserTweetsResponse,
    User as GetUserTweetsResponseUser,
};

/// A graph ql response
#[derive(Debug, serde::Deserialize)]
pub struct GraphQlResponse<T> {
    /// The data
    pub data: T,
}
