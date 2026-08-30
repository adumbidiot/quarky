use crate::PoiseContext;

/// Respond with pong
#[poise::command(slash_command)]
pub async fn ping(ctx: PoiseContext<'_>) -> anyhow::Result<()> {
    ctx.say("pong").await?;
    Ok(())
}
