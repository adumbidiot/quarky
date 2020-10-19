use log::{
    info,
    warn,
};
use serenity::{
    cache::Cache,
    client::Context,
    framework::standard::{
        macros::command,
        Args,
        CommandResult,
    },
    http::Http,
    model::channel::{
        ChannelType,
        Message,
    },
};

#[command]
#[description("Broadcast a message to robotics members")]
#[allowed_roles("bot")] //TODO: Admin only
#[usage("\"<announcement>\"")]
#[min_args(1)]
#[max_args(1)]
#[example("\"Hello! This is an announcement!\"")]
pub async fn announce(ctx: &Context, _msg: &Message, mut args: Args) -> CommandResult {
    let announcement = args.single_quoted::<String>()?;
    announce_discord(&ctx.http, &ctx.cache, &announcement).await;
    Ok(())
}

pub async fn announce_discord(http: &Http, cache: &Cache, data: &str) {
    for guild_id in cache.guilds().await.into_iter() {
        if let Some(guild) = cache.guild(guild_id).await {
            let channel = guild.channels.values().find(|channel| {
                channel.name == "announcements" && channel.kind == ChannelType::Text
            });

            if let Some(channel) = channel {
                info!(
                    r#"Announcing "{}" to discord channel "{}/{}""#,
                    data,
                    guild.name,
                    channel.name()
                );

                // Don't let one failure stop the fun
                if let Err(e) = channel.say(&http, data).await {
                    warn!(
                        "Failed to announce in channel '{}' in '{}': {}",
                        channel.name(),
                        guild.name,
                        e
                    );
                }
            }
        }
    }
}
