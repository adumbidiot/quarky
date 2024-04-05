use anyhow::Context as _;
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
    let (maybe_guild_id, maybe_channel_id) = match msg.guild(&ctx.cache) {
        Some(guild) => {
            let maybe_channel_id = guild
                .voice_states
                .get(&msg.author.id)
                .and_then(|voice_state| voice_state.channel_id);
            (Some(guild.id), maybe_channel_id)
        }
        None => (None, None),
    };

    let guild_id = match maybe_guild_id {
        Some(guild_id) => guild_id,
        None => {
            msg.channel_id
                .say(&ctx.http, "Groups and DMs not supported")
                .await?;
            return Ok(());
        }
    };

    let connect_to = match maybe_channel_id {
        Some(channel) => channel,
        None => {
            msg.reply(&ctx.http, "Not in a voice channel").await?;
            return Ok(());
        }
    };

    let manager = songbird::get(ctx)
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
            error!("{error}");
            msg.channel_id.say(&ctx.http, format!("{error}")).await?;
            return Ok(());
        }
    };

    msg.channel_id
        .say(&ctx.http, format!("Joined {}", connect_to.mention()))
        .await?;

    Ok(())
}
