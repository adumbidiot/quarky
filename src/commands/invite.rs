use crate::CommandContext;

/// Get an invite link for this bot
#[poise::command(slash_command)]
pub async fn invite(ctx: CommandContext<'_>) -> anyhow::Result<()> {
    let app_info = ctx.http().get_current_application_info().await?;

    let id = app_info.id;
    let permissions = "1341644225";
    let link = format!(
        "https://discordapp.com/oauth2/authorize?client_id={id}&scope=bot&permissions={permissions}",
    );
    ctx.say(link).await?;

    Ok(())
}
