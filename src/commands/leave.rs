use crate::CommandContext;
use log::warn;

/// Leave a voice channel
#[poise::command(slash_command)]
pub async fn leave(ctx: CommandContext<'_>) -> anyhow::Result<()> {
    let maybe_guild_id = ctx.guild_id();
    let guild_id = match maybe_guild_id {
        Some(guild_id) => guild_id,
        None => {
            ctx.say("Groups and DMs are not supported").await?;
            return Ok(());
        }
    };

    let manager = songbird::get(ctx.serenity_context())
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

        ctx.say("Left voice channel").await?;
    } else {
        ctx.say("Not in a voice channel").await?;
    }

    Ok(())
}
