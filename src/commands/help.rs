use crate::CommandContext;

/// Get help
#[poise::command(slash_command)]
pub async fn help(
    ctx: CommandContext<'_>,
    #[description = "Command to get help for"] mut command: Option<String>,
) -> anyhow::Result<()> {
    if ctx.invoked_command_name() != "help" {
        command = match command {
            Some(command) => Some(format!("{} {}", ctx.invoked_command_name(), command)),
            None => Some(ctx.invoked_command_name().to_string()),
        };
    }

    poise::builtins::help(ctx, command.as_deref(), Default::default()).await?;

    Ok(())
}
