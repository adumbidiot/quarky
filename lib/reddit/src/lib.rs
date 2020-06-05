pub mod types;

pub use crate::types::{
    Listing,
    PostHint,
    SubRedditEntry,
    SubRedditEntryData,
    SubRedditListing,
};
use hyper::{
    client::HttpConnector,
    Client as HyperClient,
    StatusCode,
};
use hyper_tls::HttpsConnector;

#[derive(Debug)]
pub enum RedditError {
    Hyper(hyper::Error),
    InvalidUri(http::uri::InvalidUri),

    InvalidStatusCode(http::StatusCode),
    NotFound,

    Json(serde_json::Error, Option<bytes::Bytes>),
}

impl From<hyper::Error> for RedditError {
    fn from(e: hyper::Error) -> Self {
        Self::Hyper(e)
    }
}

impl From<http::uri::InvalidUri> for RedditError {
    fn from(e: http::uri::InvalidUri) -> Self {
        RedditError::InvalidUri(e)
    }
}

impl RedditError {
    pub fn is_not_found(&self) -> bool {
        match self {
            RedditError::NotFound => true,
            _ => false,
        }
    }
}

pub type RedditResult<T> = Result<T, RedditError>;

pub struct Client {
    client: HyperClient<HttpsConnector<HttpConnector>, hyper::Body>,
}

impl Client {
    pub fn new() -> Self {
        let https = HttpsConnector::new();
        let client = HyperClient::builder().build::<_, hyper::Body>(https);
        Client { client }
    }

    pub async fn get_subreddit(
        &self,
        subreddit: &str,
        n: usize,
    ) -> Result<SubRedditListing, RedditError> {
        let uri = format!("https://www.reddit.com/r/{}.json?limit={}", subreddit, n).parse()?;
        let res = self.client.get(uri).await?;

        let status = res.status();
        if !status.is_success() {
            return match status {
                StatusCode::FOUND => match res.headers().get(hyper::header::LOCATION) {
                    Some(link) => {
                        let url = b"https://www.reddit.com/subreddits/search.json?";
                        if link.as_ref().starts_with(url) {
                            Err(RedditError::NotFound)
                        } else {
                            Err(RedditError::InvalidStatusCode(status))
                        }
                    }
                    None => Err(RedditError::InvalidStatusCode(status)),
                },
                _ => Err(RedditError::InvalidStatusCode(status)),
            };
        }

        let body = hyper::body::to_bytes(res).await?;

        serde_json::from_slice(&body).map_err(|e| RedditError::Json(e, Some(body)))
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
