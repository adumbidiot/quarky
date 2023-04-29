use anyhow::Context as _;
use log::{
    info,
    warn,
};
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
    let client = twitter_scraper::Client::new();
    client
        .init_session()
        .await
        .context("failed to init session")?;
    let user_response = client
        .get_user_by_screen_name(user)
        .await
        .context("failed to get user by screen name")?;
    let user_id = user_response.data.user.result.rest_id.as_str();

    info!("Twitter user id for \"{user}\" is \"{user_id}\"");

    let user_tweets = client
        .get_user_media(user_id, Some(200))
        .await
        .context("failed to get user tweets")?;
    let timeline = user_tweets.data.user.result.timeline_v2.timeline;

    let entries = timeline
        .instructions
        .iter()
        .find_map(|instruction| match instruction {
            twitter_scraper::GetUserTweetsResponseTimelineInstruction::AddEntries { entries } => {
                Some(entries)
            }
            _ => None,
        })
        .context("missing AddEntries instruction in timeline")?;
    let entries: Vec<_> = entries
        .iter()
        .filter_map(|entry| entry.entry_id.strip_prefix("tweet-"))
        .collect();

    Ok(entries
        .choose(&mut OsRng)
        .map(|tweet_id| format!("https://twitter.com/{user}/status/{tweet_id}")))
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
            warn!("No tweets retrieved for '{}'", user);
            msg.channel_id
                .say(&ctx.http, format!("No tweets retrieved for {user}"))
                .await?;
            return Ok(());
        }
        Err(e) => {
            msg.channel_id
                .say(&ctx.http, format!("Twitter Api Error: {e}"))
                .await?;
            warn!("Failed to get random tweet for {user}: {e}");
            return Ok(());
        }
    }

    Ok(())
}
