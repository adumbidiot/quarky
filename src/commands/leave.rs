use log::warn;
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
    let maybe_guild_id = ctx
        .cache
        .channel(msg.channel_id)
        .map(|channel| channel.guild_id);
    let guild_id = match maybe_guild_id {
        Some(guild_id) => guild_id,
        None => {
            msg.channel_id
                .say(&ctx.http, "Groups and DMs not supported")
                .await?;
            return Ok(());
        }
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.leave(guild_id).await {
            warn!("Failed to leave voice channel: {}", e);
        }

        if let Err(e) = manager.remove(guild_id).await {
            warn!("Failed to remove voice channel: {}", e);
        }

        msg.channel_id.say(&ctx.http, "Left voice channel").await?;
    } else {
        msg.reply(&ctx.http, "Not in a voice channel").await?;
    }

    Ok(())
}
