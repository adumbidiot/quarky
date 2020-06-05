use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

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
    pub author_flair_type: Option<String>,
    pub author_fullname: Option<String>,
    pub author_patreon_flair: Option<bool>,
    pub can_gild: bool,
    pub can_mod_post: bool,
    pub clicked: bool,
    pub contest_mode: bool,
    pub created: f64,
    pub created_utc: f64,
    pub domain: String,
    pub downs: u64,
    pub edited: Value, // Can be a bool or a f32
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
    pub parent_whitelist_status: Option<String>,
    pub permalink: String,
    pub pinned: bool,
    pub post_hint: Option<PostHint>,
    pub pwls: Option<u32>,
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
    pub thumbnail_height: Option<u32>,
    pub thumbnail_width: Option<u32>,
    pub title: String,
    pub ups: u32,
    pub url: String,
    pub visited: bool,
    pub whitelist_status: Option<String>,
    pub wls: Option<u32>,

    #[serde(flatten)]
    pub unknown: HashMap<String, Value>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum PostHint {
    #[serde(rename = "image")]
    Image,

    #[serde(rename = "link")]
    Link,

    #[serde(rename = "hosted:video")]
    HostedVideo,

    #[serde(rename = "rich:video")]
    RichVideo,

    #[serde(rename = "self")]
    DataSelf,
}
