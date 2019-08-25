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
fn leave(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
    let guild_id = match ctx.cache.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            msg.channel_id
                .say(&ctx.http, "Groups and DMs not supported")?;
            return Ok(());
        }
    };

    let manager_lock = ctx.data.read().get::<VoiceManagerKey>().cloned().unwrap();

    let mut manager = manager_lock.lock();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        manager.leave(guild_id);
        manager.remove(guild_id);
        msg.channel_id.say(&ctx.http, "Left voice channel")?;
    } else {
        msg.reply(&ctx, "Not in a voice channel")?;
    }

    Ok(())
}
