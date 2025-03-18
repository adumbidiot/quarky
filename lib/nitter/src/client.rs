use crate::{
    Error,
    Feed,
};
use reqwest::header::{
    HeaderMap,
    HeaderValue,
    ACCEPT,
    ACCEPT_LANGUAGE,
    USER_AGENT,
};
use scraper::Html;

static USER_AGENT_VALUE: HeaderValue = HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Safari/537.36");
static ACCEPT_VALUE: HeaderValue = HeaderValue::from_static("*/*");
static ACCEPT_LANGUAGE_VALUE: HeaderValue = HeaderValue::from_static("en-US,en;q=0.9");

/// A nitter client
#[derive(Debug, Clone)]
pub struct Client {
    /// The inner http client
    client: reqwest::Client,
}

impl Client {
    /// Make a new client.
    pub fn new() -> Self {
        let mut default_headers = HeaderMap::new();
        default_headers.insert(USER_AGENT, USER_AGENT_VALUE.clone());
        default_headers.insert(ACCEPT, ACCEPT_VALUE.clone());
        default_headers.insert(ACCEPT_LANGUAGE, ACCEPT_LANGUAGE_VALUE.clone());

        let client = reqwest::Client::builder()
            .default_headers(default_headers)
            .build()
            .unwrap();

        Self { client }
    }

    /// Scrape feed.
    pub async fn scrape_feed(&self, url: &str) -> Result<Feed, Error> {
        let text = self
            .client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let feed = tokio::task::spawn_blocking(move || {
            let html = Html::parse_document(text.as_str());
            Feed::from_html(&html)
        })
        .await??;

        Ok(feed)
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
