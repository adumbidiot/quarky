mod model;

pub use self::model::RssFeed;
use std::time::Duration;

/// Library error type
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// HTTP Error
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    /// Tokio Join Error
    #[error(transparent)]
    TokioJoin(#[from] tokio::task::JoinError),

    /// XML De Error
    #[error(transparent)]
    XmlDe(#[from] quick_xml::DeError),
}

/// An RSS Client
pub struct Client {
    /// The inner http client
    pub client: reqwest::Client,
}

impl Client {
    /// Make a new client
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(10))
                .build()
                .expect("failed to build client"),
        }
    }

    /// Get an rss feed
    pub async fn get_feed(&self, url: &str) -> Result<RssFeed, Error> {
        let text = self
            .client
            .get(url)
            .timeout(Duration::from_secs(20))
            .send()
            .await?
            .text()
            .await?;

        let feed = tokio::task::spawn_blocking(move || quick_xml::de::from_str(&text)).await??;

        Ok(feed)
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() {
        let client = Client::new();
        let feed = client
            .get_feed("https://nitter.poast.org/dog_rates/media/rss") // https://nitter.poast.org/dog_rates/rss
            .await
            .expect("failed to get feed");
        dbg!(feed);
    }
}
