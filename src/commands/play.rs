use crate::RedditClientKey;
use serenity::{
    client::Context,
    framework::standard::{
        macros::command,
        Args,
        CommandResult,
    },
    model::channel::Message,
};
use songbird::input::YoutubeDl;

#[command]
#[bucket("voice")]
pub async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // Piggy back off of reddit client
    let client = ctx
        .data
        .read()
        .await
        .get::<RedditClientKey>()
        .unwrap()
        .client
        .client
        .clone();

    let url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            msg.channel_id
                .say(&ctx.http, "Must provide a URL to a video or audio")
                .await?;
            return Ok(());
        }
    };

    // Validation
    if !url.starts_with("http") {
        msg.channel_id
            .say(&ctx.http, "Must provide a valid URL")
            .await?;
        return Ok(());
    }

    let maybe_guild_id = ctx
        .cache
        .channel(msg.channel_id)
        .map(|channel| channel.guild_id);
    let guild_id = match maybe_guild_id {
        Some(guild_id) => guild_id,
        None => {
            msg.channel_id
                .say(&ctx.http, "Error finding channel info")
                .await?;
            return Ok(());
        }
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler) = manager.get(guild_id) {
        let mut handler = handler.lock().await;
        let source = YoutubeDl::new(client, url);
        handler.play_only_input(source.into());

        msg.channel_id.say(&ctx.http, "Playing song").await?;
    } else {
        msg.channel_id
            .say(&ctx.http, "Not in a voice channel to play in")
            .await?;
    }

    Ok(())
}
