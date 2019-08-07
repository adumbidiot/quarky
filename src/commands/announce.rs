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
pub fn announce(ctx: &mut Context, _msg: &Message, mut args: Args) -> CommandResult {
    let announcement = args.single_quoted::<String>()?;
    announce_discord(&ctx.http, &ctx.cache.read(), &announcement)?;
    Ok(())
}

pub fn announce_discord(http: &Http, cache: &Cache, data: &str) -> CommandResult {
    for (channel_id, channel_name, guild_name) in cache
        .all_guilds()
        .iter()
        .filter_map(|&guild_id| cache.guild(guild_id))
        .filter_map(|guild| {
            let guild = guild.read();
            let channel =
                guild
                    .channels
                    .values()
                    .map(|channel| channel.read())
                    .find(|channel| {
                        channel.name == "announcements" && channel.kind == ChannelType::Text
                    })?;
            Some((
                channel.id,
                channel.name().to_string(),
                guild.name.to_string(),
            ))
        })
    {
        println!(
            r#"[INFO] Announcing "{}" to discord channel "{}/{}""#,
            data, guild_name, channel_name
        );
        channel_id.say(http, data)?;
    }

    Ok(())
}
