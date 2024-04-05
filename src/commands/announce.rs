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
