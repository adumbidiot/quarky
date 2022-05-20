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

#[command]
#[bucket("voice")]
pub async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
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

    let guild_id = match ctx.cache.guild_channel(msg.channel_id) {
        Some(channel) => channel.guild_id,
        None => {
            msg.channel_id
                .say(&ctx.http, "Error finding channel info")
                .await?;
            return Ok(());
        }
    };

    let data_lock = ctx.data.read().await;
    let client = data_lock
        .get::<RedditClientKey>()
        .expect("missing reddit client")
        .client
        .client
        .clone();
    drop(data_lock);
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler) = manager.get(guild_id) {
        let mut handler = handler.lock().await;
        let source = songbird::input::Input::from(songbird::input::YoutubeDl::new(client, url));
        handler.play_source(source);

        msg.channel_id.say(&ctx.http, "Playing song").await?;
    } else {
        msg.channel_id
            .say(&ctx.http, "Not in a voice channel to play in")
            .await?;
    }

    Ok(())
}
