use crate::CommandContext;

/// Respond with pong
#[poise::command(slash_command)]
pub async fn ping(ctx: CommandContext<'_>) -> anyhow::Result<()> {
    ctx.say("pong").await?;
    Ok(())
}
