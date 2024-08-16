use crate::CommandContext;

/// Stop playing audio
#[poise::command(slash_command)]
pub async fn stop(ctx: CommandContext<'_>) -> anyhow::Result<()> {
    let maybe_guild_id = ctx.guild_id();

    let guild_id = match maybe_guild_id {
        Some(guild_id) => guild_id,
        None => {
            ctx.say("Error finding channel info").await?;
            return Ok(());
        }
    };

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler) = manager.get(guild_id) {
        handler.lock().await.stop();
        ctx.say("Stopped").await?;
    } else {
        ctx.say("Not in a voice channel").await?;
    }

    Ok(())
}
