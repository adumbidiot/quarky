use crate::VoiceManagerKey;
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
pub async fn stop(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild_id = match ctx.cache.guild_channel(msg.channel_id).await {
        Some(channel) => channel.guild_id,
        None => {
            msg.channel_id
                .say(&ctx.http, "Error finding channel info")
                .await?;
            return Ok(());
        }
    };

    let manager_lock = ctx
        .data
        .read()
        .await
        .get::<VoiceManagerKey>()
        .cloned()
        .unwrap();
    let mut manager = manager_lock.lock().await;

    if let Some(handler) = manager.get_mut(guild_id) {
        handler.stop();
        msg.channel_id.say(&ctx.http, "Stopped").await?;
    } else {
        msg.channel_id
            .say(&ctx.http, "Not in a voice channel")
            .await?;
    }

    Ok(())
}
