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
async fn leave(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild_id = match ctx.cache.guild_channel(msg.channel_id).await {
        Some(channel) => channel.guild_id,
        None => {
            msg.channel_id
                .say(&ctx.http, "Groups and DMs not supported")
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
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        manager.leave(guild_id);
        manager.remove(guild_id);
        msg.channel_id.say(&ctx.http, "Left voice channel").await?;
    } else {
        msg.reply(&ctx.http, "Not in a voice channel").await?;
    }

    Ok(())
}
