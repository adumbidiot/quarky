use crate::CommandContext;
use anyhow::Context as _;
use log::error;
use serenity::prelude::Mentionable;

/// Join a voice channel
#[poise::command(slash_command)]
pub async fn join(ctx: CommandContext<'_>) -> anyhow::Result<()> {
    let (maybe_guild_id, maybe_channel_id) = match ctx.guild() {
        Some(guild) => {
            let maybe_channel_id = guild
                .voice_states
                .get(&ctx.author().id)
                .and_then(|voice_state| voice_state.channel_id);
            (Some(guild.id), maybe_channel_id)
        }
        None => (None, None),
    };

    let guild_id = match maybe_guild_id {
        Some(guild_id) => guild_id,
        None => {
            ctx.say("Groups and DMs not supported").await?;
            return Ok(());
        }
    };

    let connect_to = match maybe_channel_id {
        Some(channel) => channel,
        None => {
            ctx.say("Not in a voice channel").await?;
            return Ok(());
        }
    };

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("missing songbird voice client")
        .clone();

    let _call = match manager
        .join(guild_id, connect_to)
        .await
        .context("failed to join voice channel")
    {
        Ok(call) => call,
        Err(error) => {
            error!("{error:?}");
            ctx.say(format!("{error:?}")).await?;
            return Ok(());
        }
    };

    ctx.say(format!("Joined {}", connect_to.mention())).await?;

    Ok(())
}
