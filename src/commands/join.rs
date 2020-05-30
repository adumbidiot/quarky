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
async fn join(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild = match msg.guild(&ctx.cache).await {
        Some(guild) => guild,
        None => {
            msg.channel_id
                .say(&ctx.http, "Groups and DMs not supported")
                .await?;
            return Ok(());
        }
    };

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            msg.reply(&ctx.http, "Not in a voice channel").await?;
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
    if manager.join(guild.id, connect_to).is_some() {
        msg.channel_id
            .say(&ctx.http, &format!("Joined {}", connect_to.mention()))
            .await?;
    } else {
        msg.channel_id
            .say(&ctx.http, "Error joining the channel")
            .await?;
    }

    Ok(())
}
