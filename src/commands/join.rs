use crate::VoiceManagerKey;
use serenity::{
    client::Context,
    framework::standard::{
        macros::command,
        Args,
        CommandResult,
    },
    model::{
        channel::Message,
        misc::Mentionable,
    },
};

#[command]
#[bucket("voice")]
fn join(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild = match msg.guild(&ctx.cache) {
        Some(guild) => guild,
        None => {
            msg.channel_id
                .say(&ctx.http, "Groups and DMs not supported")?;
            return Ok(());
        }
    };

    let guild_id = guild.read().id;

    let channel_id = guild
        .read()
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            msg.reply(&ctx, "Not in a voice channel")?;
            return Ok(());
        }
    };

    let manager_lock = ctx.data.read().get::<VoiceManagerKey>().cloned().unwrap();

    let mut manager = manager_lock.lock();
    if manager.join(guild_id, connect_to).is_some() {
        msg.channel_id
            .say(&ctx.http, &format!("Joined {}", connect_to.mention()))?;
    } else {
        msg.channel_id.say(&ctx.http, "Error joining the channel")?;
    }

    Ok(())
}
