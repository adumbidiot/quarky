mod client;
pub mod model;

pub use crate::{
    client::Client,
    model::{
        GetUserByScreenNameResponse,
        GetUserTweetsResponse,
        GetUserTweetsResponseTimelineInstruction,
        GraphQlResponse,
    },
};

/// Library error
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// http error
    #[error("http error")]
    Reqwest(#[from] reqwest::Error),

    /// Failed to join tokio task
    #[error(transparent)]
    TokioJoin(#[from] tokio::task::JoinError),

    /// Cookie Store error
    #[error("cookie store error")]
    CookieStore(#[from] cookie_store::CookieError),

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
    //use super::*;

    // Twitter seems to have broken this endpoint.
    // Check out later to see if it can be fixed.
    /*
    #[tokio::test]
    async fn get_user_tweets() {
        let client = Client::new();

        client.init_session().await.expect("failed to init session");

        let user = client
            .get_user_by_screen_name("dog_rates")
            .await
            .expect("failed to get user by screen name");

        let user_tweets = client
            .get_user_tweets(user.data.user.result.rest_id.as_str(), Some(200), None)
            .await
            .expect("failed to get user tweets");
        dbg!(user_tweets.data);
    }
    */

    // Twitter seems to have removed this endpoint.
    // Keep it around for a bit in case they change their minds.
    /*
    #[tokio::test]
    async fn get_user_media() {
        let client = Client::new();

        client.init_session().await.expect("failed to init session");

        let user = client
            .get_user_by_screen_name("dog_rates")
            .await
            .expect("failed to get user by screen name");

        let user_media = client
            .get_user_media(user.data.user.result.rest_id.as_str(), Some(200))
            .await
            .expect("failed to get user media");
        dbg!(user_media.data);
    }
    */
}
