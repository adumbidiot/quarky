use log::error;
use serenity::{
    client::Context,
    framework::standard::{
        macros::command,
        Args,
        CommandResult,
    },
    model::channel::Message,
    prelude::Mentionable,
};

#[command]
#[bucket("voice")]
async fn join(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild = match msg.guild(&ctx.cache) {
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

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let (_call, result) = manager.join(guild.id, connect_to).await;

    match result {
        Ok(()) => {
            msg.channel_id
                .say(&ctx.http, format!("Joined {}", connect_to.mention()))
                .await?;
        }
        Err(e) => {
            msg.channel_id
                .say(&ctx.http, "Error joining the voice channel")
                .await?;
            error!("Failed to join the voice channel: {}", e);
        }
    }

    Ok(())
}
