mod get_user_by_screen_name;
mod get_user_tweets;

pub use self::{
    get_user_by_screen_name::GetUserByScreenNameResponse,
    get_user_tweets::{
        GetUserTweetsResponse,
        TimelineInstruction as GetUserTweetsResponseTimelineInstruction,
    },
};

/// A graph ql response
#[derive(Debug, serde::Deserialize)]
pub struct GraphQlResponse<T> {
    /// The data
    pub data: T,
}

/// Get a user field
#[derive(Debug, serde::Deserialize)]
pub struct UserField<T> {
    pub user: T,
}

/// Get a result field
#[derive(Debug, serde::Deserialize)]
pub struct ResultField<T> {
    pub result: T,
}
