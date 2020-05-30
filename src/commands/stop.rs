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
fn stop(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild_id = match ctx.cache.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            msg.channel_id
                .say(&ctx.http, "Error finding channel info")?;
            return Ok(());
        }
    };

    let manager_lock = ctx.data.read().get::<VoiceManagerKey>().cloned().unwrap();
    let mut manager = manager_lock.lock();

    if let Some(handler) = manager.get_mut(guild_id) {
        handler.stop();
        msg.channel_id.say(&ctx.http, "Stopped")?;
    } else {
        msg.channel_id.say(&ctx.http, "Not in a voice channel")?;
    }

    Ok(())
}
