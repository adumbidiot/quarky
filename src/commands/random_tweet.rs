use crate::RssClientKey;
use anyhow::Context as _;
use log::warn;
use rand::{
    prelude::SliceRandom,
    rngs::OsRng,
};
use serenity::{
    client::Context,
    framework::standard::{
        macros::command,
        Args,
        CommandResult,
    },
    model::channel::Message,
};
use std::{
    future::Future,
    time::Duration,
};

async fn retry<FN, T, E, FU>(mut func: FN, max_tries: u64) -> Result<T, E>
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
                tokio::time::sleep(Duration::from_secs(num_try)).await;
            }
            Err(error) => {
                break Err(error);
            }
        }
    }
}

pub async fn get_random_tweet_url(
    client: &rss_client::Client,
    user: &str,
) -> anyhow::Result<Option<String>> {
    let url = format!("https://twiiit.com/{user}/media/rss");
    let feed = retry(|| client.get_feed(&url), 10)
        .await
        .with_context(|| format!("failed to get nitter rss feed for \"{user}\""))?;

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
        .choose(&mut OsRng)
        .map(|tweet_id| format!("https://fxtwitter.com/{user}/status/{tweet_id}"));

    Ok(url)
}

#[command("random-tweet")]
#[description = "Get a random tweet for a user"]
#[min_args(1)]
#[max_args(1)]
pub async fn random_tweet(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let user = args.single::<String>().unwrap();
    let rss_client = ctx.data.read().await.get::<RssClientKey>().unwrap().clone();

    match get_random_tweet_url(&rss_client, &user).await {
        Ok(Some(url)) => {
            msg.channel_id.say(&ctx.http, url).await?;
        }
        Ok(None) => {
            warn!("No tweets retrieved for \"{user}\"");
            msg.channel_id
                .say(&ctx.http, format!("No tweets retrieved for \"{user}\""))
                .await?;
            return Ok(());
        }
        Err(error) => {
            msg.channel_id
                .say(&ctx.http, format!("Twitter Api Error: {error:?}"))
                .await?;
            warn!("Failed to get random tweet for \"{user}\": {error:?}");
            return Ok(());
        }
    }

    Ok(())
}
