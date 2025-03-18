mod client;
pub mod model;

pub use self::{
    client::Client,
    model::{
        feed::FromHtmlError as InvalidFeedError,
        Feed,
    },
};

/// The library error type
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Http error
    #[error("http error")]
    Reqwest(#[from] reqwest::Error),

    /// Tokio join
    #[error("tokio join error")]
    TokioJoin(#[from] tokio::task::JoinError),

    /// Failed to parse a feed
    #[error("invalid feed")]
    InvalidFeed(#[from] InvalidFeedError),
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn it_works() {
        let url = "https://nitter.net/dog_rates/media";

        let client = Client::new();
        let feed = client
            .scrape_feed(url)
            .await
            .expect("failed to scrape feed");
        assert!(!feed.timeline_items.is_empty());
    }
}
