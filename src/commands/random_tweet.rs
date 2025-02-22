use crate::{
    CommandContext,
    RssClientKey,
};
use anyhow::Context as _;
use log::warn;
use rand::prelude::IndexedRandom;
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

pub async fn get_random_tweet_url(
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
        .filter_map(|item| {
            // https://<domain>/<user>/status/<tweet_id>#m

            let mut path_iter = item.link.link.path_segments()?;
            if path_iter.next()? != user {
                return None;
            }

            if path_iter.next()? != "status" {
                return None;
            }

            path_iter.next()
        })
        .collect();

    // Twitter embeds broke as of 10/10/2023.
    // Use https://github.com/FixTweet/FixTweet instead for embedding.
    let url = entries
        .choose(&mut rand::rng())
        .map(|tweet_id| format!("https://fxtwitter.com/{user}/status/{tweet_id}"));

    Ok(url)
}

/// Get a random tweet for a user
#[poise::command(slash_command)]
pub async fn random_tweet(
    ctx: CommandContext<'_>,
    #[description = "The user"] user: String,
) -> anyhow::Result<()> {
    let rss_client = ctx
        .serenity_context()
        .data
        .read()
        .await
        .get::<RssClientKey>()
        .unwrap()
        .clone();

    ctx.defer().await?;

    match get_random_tweet_url(&rss_client, &user)
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
