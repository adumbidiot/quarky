mod client;
pub mod model;

pub use crate::{
    client::Client,
    model::{
        GetUserByScreenNameResponse,
        GetUserTweetsResponse,
        GraphQlResponse,
    },
};

/// Library error
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// http error
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    /// Failed to join tokio task
    #[error(transparent)]
    TokioJoin(#[from] tokio::task::JoinError),

    /// Cookie error
    #[error(transparent)]
    Cookie(#[from] cookie_store::CookieError),

    /// Missing guest token
    #[error("missing guest token")]
    MissingGuestToken,

    /// Json error
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    /// Url error
    #[error(transparent)]
    ParseUrl(#[from] url::ParseError),
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn get_user_tweets() {
        let client = Client::new();

        client.init_session().await.expect("failed to init session");

        let user = client
            .get_user_by_screen_name("dog_rates")
            .await
            .expect("failed to get user by screen name");

        let user_tweets = client
            .get_user_tweets(user.data.user.result.rest_id.as_str())
            .await
            .expect("failed to get user tweets");
        dbg!(user_tweets.data);
    }
}
