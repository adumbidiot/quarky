use crate::TwitterTokenKey;
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

#[command("random-tweet")]
#[description = "Get a random tweet for a user"]
#[min_args(1)]
#[max_args(1)]
pub async fn random_tweet(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let user = args.single::<String>().unwrap();

    let data_lock = ctx.data.read().await;
    let token = data_lock.get::<TwitterTokenKey>().unwrap().clone();
    drop(data_lock);

    let timeline = egg_mode::tweet::user_timeline(user.clone(), false, false, &token);

    let (_timeline, feed) = match timeline.start().await {
        Ok(data) => data,
        Err(e) => {
            msg.channel_id
                .say(&ctx.http, format!("Twitter Api Error: {}", e))
                .await?;
            eprintln!("[WARN] Failed to get random tweet for {}: {}", user, e);
            return Ok(());
        }
    };

    let maybe_tweet = feed.choose(&mut rand::thread_rng());
    let tweet = match maybe_tweet {
        Some(tweet) => tweet,
        None => {
            eprintln!("[WARN] No tweets retrieved for '{}'", user);
            msg.channel_id
                .say(&ctx.http, format!("No tweets retrieved for {}", user))
                .await?;
            return Ok(());
        }
    };

    msg.channel_id
        .say(
            &ctx.http,
            format!("https://twitter.com/{}/status/{}", user, tweet.id),
        )
        .await?;

    Ok(())
}
