use crate::TwitterTokenKey;
use log::warn;
use rand::prelude::SliceRandom;
use serenity::{
    client::Context,
    framework::standard::{
        macros::command,
        Args,
        CommandResult,
    },
    model::channel::Message,
};

pub async fn get_random_tweet_url(
    twitter_token: &egg_mode::auth::Token,
    user: &str,
) -> Result<Option<String>, egg_mode::error::Error> {
    let timeline = egg_mode::tweet::user_timeline(user.to_string(), false, false, &twitter_token);

    let (_timeline, feed) = timeline.with_page_size(200).start().await?;

    let mut rng = rand::thread_rng();
    Ok(feed
        .choose(&mut rng)
        .map(|tweet| format!("https://twitter.com/{}/status/{}", user, tweet.id)))
}

#[command("random-tweet")]
#[description = "Get a random tweet for a user"]
#[min_args(1)]
#[max_args(1)]
pub async fn random_tweet(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let user = args.single::<String>().unwrap();

    let data_lock = ctx.data.read().await;
    let token = data_lock.get::<TwitterTokenKey>().unwrap().clone();
    drop(data_lock);

    match get_random_tweet_url(&token, &user).await {
        Ok(Some(url)) => {
            msg.channel_id.say(&ctx.http, url).await?;
        }
        Ok(None) => {
            warn!("No tweets retrieved for '{}'", user);
            msg.channel_id
                .say(&ctx.http, format!("No tweets retrieved for {}", user))
                .await?;
            return Ok(());
        }
        Err(e) => {
            msg.channel_id
                .say(&ctx.http, format!("Twitter Api Error: {}", e))
                .await?;
            warn!("Failed to get random tweet for {}: {}", user, e);
            return Ok(());
        }
    }

    Ok(())
}
