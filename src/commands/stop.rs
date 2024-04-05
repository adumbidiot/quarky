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
        handler.lock().await.stop();
        msg.channel_id.say(&ctx.http, "Stopped").await?;
    } else {
        msg.channel_id
            .say(&ctx.http, "Not in a voice channel")
            .await?;
    }

    Ok(())
}
