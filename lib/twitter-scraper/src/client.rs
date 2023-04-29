use crate::{
    Error,
    GetUserByScreenNameResponse,
    GetUserTweetsResponse,
    GraphQlResponse,
};
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::header::HeaderValue;
use serde_json::json;
use std::sync::Arc;
use url::Url;

const USER_AGENT_STR: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.0.0.0 Safari/537.36";
static AUTHORIZATION_VALUE: HeaderValue = HeaderValue::from_static("Bearer AAAAAAAAAAAAAAAAAAAAANRILgAAAAAAnNwIzUejRCOuH5E6I8xnZz4puTs%3D1Zv7ttfk8LF81IUq16cHjhLTvJu4FA33AGWWjCpTnA");

/// The client
#[derive(Debug, Clone)]
pub struct Client {
    /// The inner http client
    pub client: reqwest::Client,

    cookie_store: Arc<reqwest_cookie_store::CookieStoreMutex>,
}

impl Client {
    /// Make a new client
    pub fn new() -> Self {
        let cookie_store = reqwest_cookie_store::CookieStore::default();
        let cookie_store = reqwest_cookie_store::CookieStoreMutex::new(cookie_store);
        let cookie_store = std::sync::Arc::new(cookie_store);

        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT_STR)
            .cookie_provider(cookie_store.clone())
            .build()
            .expect("failed to build client");

        Self {
            client,
            cookie_store,
        }
    }

    /// Init a session by GETing the home page
    pub async fn init_session(&self) -> Result<(), Error> {
        static REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new("document.cookie=\"(gt=.*?)\"").unwrap());

        let home_url = Url::parse("https://twitter.com").unwrap();
        let text = self
            .client
            .get(home_url.as_str())
            .send()
            .await?
            .text()
            .await?;

        let gt_cookie_str = tokio::task::spawn_blocking(move || {
            REGEX
                .captures(&text)
                .and_then(|captures| captures.get(1))
                .map(|capture| capture.as_str().to_string())
        })
        .await?;

        if let Some(gt_cookie_str) = gt_cookie_str {
            let mut cookie_store = self.cookie_store.lock().expect("cookie store poisoned");
            cookie_store.parse(&gt_cookie_str, &home_url)?;
        }

        Ok(())
    }

    async fn make_graphql_request<V, F, T>(
        &self,
        name: &str,
        query_hash: &str,
        variables: &V,
        features: &F,
    ) -> Result<GraphQlResponse<T>, Error>
    where
        V: serde::Serialize,
        F: serde::Serialize,
        T: serde::de::DeserializeOwned,
    {
        let gt = self
            .cookie_store
            .lock()
            .expect("cookie store poisoned")
            .get("twitter.com", "/", "gt")
            .ok_or(Error::MissingGuestToken)?
            .value()
            .to_string();

        let variables = serde_json::to_string(&variables)?;
        let features = serde_json::to_string(&features)?;

        let url = Url::parse_with_params(
            &format!("https://twitter.com/i/api/graphql/{query_hash}/{name}"),
            &[
                ("variables", variables.as_str()),
                ("features", features.as_str()),
            ],
        )?;
        let url = url.as_str();

        let json = self
            .client
            .get(url)
            .header(reqwest::header::AUTHORIZATION, AUTHORIZATION_VALUE.clone())
            .header("x-guest-token", gt)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(json)
    }

    /// Get the user by screen name
    pub async fn get_user_by_screen_name(
        &self,
        name: &str,
    ) -> Result<GraphQlResponse<GetUserByScreenNameResponse>, Error> {
        const QUERY_HASH: &str = "sLVLhk0bGj3MVFEKTdax1w";

        let variables = json!({
            "screen_name": name,
            "withSafetyModeUserFields": true,
        });
        let features = json!({
           "blue_business_profile_image_shape_enabled": true,
           "responsive_web_graphql_exclude_directive_enabled": true,
           "verified_phone_label_enabled": false,
           "responsive_web_graphql_skip_user_profile_image_extensions_enabled": false,
           "responsive_web_graphql_timeline_navigation_enabled": true,
        });

        self.make_graphql_request("UserByScreenName", QUERY_HASH, &variables, &features)
            .await
    }

    /// Get user tweets
    pub async fn get_user_tweets(
        &self,
        user_id: &str,
        count: Option<usize>,
    ) -> Result<GraphQlResponse<GetUserTweetsResponse>, Error> {
        const QUERY_HASH: &str = "CdG2Vuc1v6F5JyEngGpxVw";

        let variables = json!({
            "userId": user_id,
            "count": count.unwrap_or(40),
            "includePromotedContent": true,
            "withQuickPromoteEligibilityTweetFields": true,
            "withVoice": true,
            "withV2Timeline": true,
        });
        let features = json!({
            "blue_business_profile_image_shape_enabled": true,
            "responsive_web_graphql_exclude_directive_enabled": true,
            "verified_phone_label_enabled": false,
            "responsive_web_graphql_timeline_navigation_enabled": true,
            "responsive_web_graphql_skip_user_profile_image_extensions_enabled": false,
            "tweetypie_unmention_optimization_enabled": true,
            "vibe_api_enabled": true,
            "responsive_web_edit_tweet_api_enabled": true,
            "graphql_is_translatable_rweb_tweet_is_translatable_enabled": true,
            "view_counts_everywhere_api_enabled": true,
            "longform_notetweets_consumption_enabled": true,
            "tweet_awards_web_tipping_enabled": false,
            "freedom_of_speech_not_reach_fetch_enabled": true,
            "standardized_nudges_misinfo": true,
            "tweet_with_visibility_results_prefer_gql_limited_actions_policy_enabled": false,
            "interactive_text_enabled": true,
            "responsive_web_text_conversations_enabled": false,
            "longform_notetweets_rich_text_read_enabled": true,
            "responsive_web_enhance_cards_enabled": false,
        });

        self.make_graphql_request("UserTweets", QUERY_HASH, &variables, &features)
            .await
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
