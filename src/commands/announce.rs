use crate::CommandContext;
use log::{
    info,
    warn,
};
use serenity::{
    cache::Cache,
    http::Http,
    model::channel::ChannelType,
};

/*
#[command]
#[allowed_roles("bot")] //TODO: Admin only
pub async fn announce(ctx: &Context, _msg: &Message, mut args: Args) -> CommandResult {
    let announcement = args.single_quoted::<String>()?;
    announce_discord(&ctx.http, &ctx.cache, &announcement).await;
    Ok(())
}
*/

async fn has_bot_role(ctx: CommandContext<'_>) -> anyhow::Result<bool> {
    let (guild_id, role_id) = {
        let guild = match ctx.guild() {
            Some(guild) => guild,
            None => {
                return Ok(false);
            }
        };

        let role_id = guild.roles.iter().find_map(|(id, role)| {
            if role.name != "bot" {
                return None;
            }

            Some(*id)
        });

        let role_id = match role_id {
            Some(role_id) => role_id,
            None => {
                return Ok(false);
            }
        };

        (guild.id, role_id)
    };

    let has_role = ctx.author().has_role(ctx.http(), guild_id, role_id).await?;

    Ok(has_role)
}

/// Broadcast a message to robotics members
#[poise::command(slash_command, check = "has_bot_role")]
pub async fn announce(
    ctx: CommandContext<'_>,
    #[description = "The announcement"] announcement: String,
) -> anyhow::Result<()> {
    let serenity_context = ctx.serenity_context();
    announce_discord(
        &serenity_context.http,
        &serenity_context.cache,
        &announcement,
    )
    .await;
    Ok(())
}

pub async fn announce_discord(http: &Http, cache: &Cache, data: &str) {
    for guild_id in cache.guilds().into_iter() {
        let (guild_name, channel_id, channel_name) = match cache.guild(guild_id) {
            Some(guild) => {
                let guild_name = guild.name.to_string();

                let maybe_channel = guild.channels.values().find(|channel| {
                    channel.name == "announcements" && channel.kind == ChannelType::Text
                });
                let channel = match maybe_channel {
                    Some(channel) => channel,
                    None => continue,
                };
                let channel_name = channel.name().to_string();

                (guild_name, channel.id, channel_name)
            }
            None => continue,
        };

        info!("Announcing \"{data}\" to discord channel \"{guild_name}/{channel_name}\"");

        // Don't let one failure stop the fun
        if let Err(error) = channel_id.say(&http, data).await {
            warn!("Failed to announce in channel \"{channel_name}\" in \"{guild_name}\": {error}");
        }
    }
}
