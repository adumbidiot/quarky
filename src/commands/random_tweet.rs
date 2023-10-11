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

pub async fn get_random_tweet_url(user: &str) -> anyhow::Result<Option<String>> {
    // TODO: Cache?
    let client = rss_client::Client::new();
    let feed = client
        .get_feed(&format!("https://nitter.poast.org/{user}/media/rss"))
        .await
        .context("failed to get feed")?;

    let entries: Vec<_> = feed
        .channel
        .item
        .iter()
        .filter_map(|item| {
            // https://nitter.poast.org/<user>/status/<tweet_id>#m

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

    match get_random_tweet_url(&user).await {
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
