use hyper::{
    client::HttpConnector,
    rt::{
        Future,
        Stream,
    },
    Client as HyperClient,
    StatusCode,
};
use hyper_tls::HttpsConnector;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug)]
pub enum RedditError {
    Network,
    InvalidStatusCode(u16),
    NotFound,
    JsonError(serde_json::Error, Option<bytes::Bytes>),
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
    handle: HyperClient<HttpsConnector<HttpConnector>, hyper::Body>,
}

impl Client {
    pub fn new() -> Self {
        let dns_threads = 4;
        let https = HttpsConnector::new(dns_threads).unwrap();
        let handle = HyperClient::builder().build::<_, hyper::Body>(https);
        Client { handle }
    }

    pub fn get_subreddit(
        &self,
        subreddit: &str,
    ) -> impl Future<Item = SubRedditListing, Error = RedditError> {
        let uri = format!("https://www.reddit.com/r/{}.json", subreddit)
            .parse()
            .unwrap();

        self.handle
            .get(uri)
            .map_err(|_| RedditError::Network)
            .and_then(|res| {
                let status = res.status();
                if !status.is_success() {
                    return match status {
                        StatusCode::FOUND => match res.headers().get(hyper::header::LOCATION) {
                            Some(link) => {
                                let url = b"https://www.reddit.com/subreddits/search.json?";
                                if link.as_ref().starts_with(url) {
                                    Err(RedditError::NotFound)
                                } else {
                                    Err(RedditError::InvalidStatusCode(status.as_u16()))
                                }
                            }
                            None => Err(RedditError::InvalidStatusCode(status.as_u16())),
                        },
                        _ => Err(RedditError::InvalidStatusCode(status.as_u16())),
                    };
                }
                Ok(res)
            })
            .and_then(|res| res.into_body().concat2().map_err(|_| RedditError::Network))
            .and_then(|body| {
                serde_json::from_slice(&body)
                    .map_err(|e| RedditError::JsonError(e, Some(body.into_bytes())))
            })
    }
}

// From https://github.com/reddit-archive/reddit/wiki/json
#[derive(Debug, Deserialize)]
pub struct Listing<T> {
    pub before: Option<String>,
    pub after: Option<String>,
    pub modhash: String,
    pub children: Vec<T>,

    #[serde(flatten)]
    pub unknown: HashMap<String, Value>,
}

// Hand-Made
#[derive(Debug, Deserialize)]
pub struct SubRedditListing {
    pub kind: String,
    pub data: Listing<SubRedditEntry>,

    #[serde(flatten)]
    pub unknown: HashMap<String, Value>,
}

#[derive(Debug, Deserialize)]
pub struct SubRedditEntry {
    pub kind: String,
    pub data: SubRedditEntryData,

    #[serde(flatten)]
    pub unknown: HashMap<String, Value>,
}

#[derive(Debug, Deserialize)]
pub struct SubRedditEntryData {
    pub archived: bool,
    pub author: String,
    pub author_flair_css_class: Option<String>,
    pub author_flair_template_id: Option<String>,
    pub author_flair_text: Option<String>,
    pub author_flair_text_color: Option<String>,
    pub author_flair_type: String,
    pub author_fullname: String,
    pub author_patreon_flair: bool,
    pub can_gild: bool,
    pub can_mod_post: bool,
    pub clicked: bool,
    pub contest_mode: bool,
    pub created: f64,
    pub created_utc: f64,
    pub domain: String,
    pub downs: u64,
    pub edited: bool,
    pub gilded: u32,
    pub hidden: bool,
    pub hide_score: bool,
    pub id: String,
    pub is_crosspostable: bool,
    pub is_meta: bool,
    pub is_original_content: bool,
    pub is_reddit_media_domain: bool,
    pub is_robot_indexable: bool,
    pub is_self: bool,
    pub is_video: bool,
    pub link_flair_text_color: String,
    pub link_flair_type: String,
    pub locked: bool,
    pub media_only: bool,
    pub name: String,
    pub no_follow: bool,
    pub num_comments: u32,
    pub num_crossposts: u32,
    pub over_18: bool,
    pub parent_whitelist_status: String,
    pub permalink: String,
    pub pinned: bool,
    pub post_hint: PostHint,
    pub pwls: u32,
    pub quarantine: bool,
    pub saved: bool,
    pub score: u32,
    pub send_replies: bool,
    pub spoiler: bool,
    pub stickied: bool,
    pub subreddit: String,
    pub subreddit_id: String,
    pub subreddit_name_prefixed: String,
    pub subreddit_subscribers: u64,
    pub subreddit_type: String,
    pub suggested_sort: Option<String>,
    pub thumbnail: String,
    pub thumbnail_height: u32,
    pub thumbnail_width: u32,
    pub title: String,
    pub ups: u32,
    pub url: String,
    pub visited: bool,
    pub whitelist_status: String,
    pub wls: u32,

    #[serde(flatten)]
    pub unknown: HashMap<String, Value>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum PostHint {
    #[serde(rename = "image")]
    Image,

    #[serde(rename = "link")]
    Link,
}
