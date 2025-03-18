use crate::{
    CommandContext,
    NitterClientKey,
    RssClientKey,
};
use anyhow::Context as _;
use log::warn;
use rand::prelude::IndexedRandom;
use reqwest::Url;
use rss_client::RssFeed;
use std::{
    future::Future,
    time::Duration,
};

async fn retry<FN, T, E, FU>(mut func: FN, max_tries: u32) -> Result<T, E>
where
    FN: FnMut() -> FU,
    FU: Future<Output = Result<T, E>>,
    E: std::error::Error,
{
    let mut num_try = 0;
    loop {
        let future = (func)();
        match future.await {
            Ok(value) => {
                break Ok(value);
            }
            Err(error) if num_try < max_tries => {
                warn!("{error}");

                num_try += 1;
                tokio::time::sleep(Duration::from_secs(2_u64.pow(num_try))).await;
            }
            Err(error) => {
                break Err(error);
            }
        }
    }
}

async fn get_nitter_feed(
    client: &rss_client::Client,
    host: &str,
    user: &str,
) -> anyhow::Result<RssFeed> {
    let url = format!("{host}/{user}/media/rss");
    retry(|| client.get_feed(&url), 3)
        .await
        .with_context(|| format!("failed to get nitter rss feed for \"{user}\" from \"{host}\""))
}

fn extract_id_from_nitter_url<'a>(url: &'a Url, user: &str) -> Option<&'a str> {
    // https://<domain>/<user>/status/<tweet_id>#m

    let mut path_iter = url.path_segments()?;
    if path_iter.next()? != user {
        return None;
    }

    if path_iter.next()? != "status" {
        return None;
    }

    path_iter.next()
}

async fn get_random_tweet_id_nitter_rss(
    client: &rss_client::Client,
    user: &str,
) -> anyhow::Result<Option<String>> {
    let feed_result = get_nitter_feed(client, "https://nitter.privacydev.net", user).await;

    let feed = match feed_result {
        Ok(feed) => feed,
        Err(_error) => get_nitter_feed(client, "https://nitter.poast.org", user).await?,
    };

    let entries: Vec<_> = feed
        .channel
        .item
        .iter()
        .filter_map(|item| extract_id_from_nitter_url(&item.link.link, user))
        .collect();

    Ok(entries.choose(&mut rand::rng()).map(|v| v.to_string()))
}

async fn get_random_tweet_id_nitter_scrape(
    client: &nitter::Client,
    user: &str,
) -> anyhow::Result<Option<String>> {
    let base_url = Url::parse("https://nitter.net/")?;

    let url = format!("https://nitter.net/{user}/media");
    let feed = client.scrape_feed(&url).await?;
    let entries: Vec<_> = feed
        .timeline_items
        .iter()
        .filter_map(|item| {
            let url = Url::options()
                .base_url(Some(&base_url))
                .parse(&item.link)
                .ok()?;
            extract_id_from_nitter_url(&url, user).map(|v| v.to_string())
        })
        .collect();

    Ok(entries.choose(&mut rand::rng()).cloned())
}

pub async fn get_random_tweet_url(
    rss_client: &rss_client::Client,
    nitter_client: &nitter::Client,
    user: &str,
) -> anyhow::Result<Option<String>> {
    let mut tweet_id_result = Err(anyhow::Error::msg("failed to get random tweet"));
    if tweet_id_result.is_err() {
        tweet_id_result = get_random_tweet_id_nitter_rss(rss_client, user).await;
    }
    if tweet_id_result.is_err() {
        tweet_id_result = get_random_tweet_id_nitter_scrape(nitter_client, user).await
    }

    let maybe_tweet_id = tweet_id_result?;

    // Twitter embeds broke as of 10/10/2023.
    // Use https://github.com/FixTweet/FixTweet instead for embedding.
    let url =
        maybe_tweet_id.map(|tweet_id| format!("https://fxtwitter.com/{user}/status/{tweet_id}"));

    Ok(url)
}

/// Get a random tweet for a user
#[poise::command(slash_command)]
pub async fn random_tweet(
    ctx: CommandContext<'_>,
    #[description = "The user"] user: String,
) -> anyhow::Result<()> {
    let rss_client;
    let nitter_client;
    {
        let data = ctx.serenity_context().data.read().await;

        rss_client = data.get::<RssClientKey>().unwrap().clone();
        nitter_client = data.get::<NitterClientKey>().unwrap().clone();
    }

    ctx.defer().await?;

    match get_random_tweet_url(&rss_client, &nitter_client, &user)
        .await
        .with_context(|| format!("failed to get random tweet for \"{user}\""))
        .and_then(|maybe_url| {
            maybe_url.with_context(|| format!("no tweets retrieved for \"{user}\""))
        }) {
        Ok(url) => {
            ctx.say(url).await?;
        }
        Err(error) => {
            warn!("{error:?}");
            ctx.say(format!("{error:?}")).await?;
            return Ok(());
        }
    }

    Ok(())
}
