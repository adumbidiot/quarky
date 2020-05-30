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
pub fn announce(ctx: &Context, _msg: &Message, mut args: Args) -> CommandResult {
    let announcement = args.single_quoted::<String>()?;
    announce_discord(&ctx.http, &ctx.cache.read(), &announcement)?;
    Ok(())
}

pub fn announce_discord(http: &Http, cache: &Cache, data: &str) -> CommandResult {
    for guild in cache
        .all_guilds()
        .iter()
        .filter_map(|&guild_id| cache.guild(guild_id))
    {
        let guild = guild.read();
        let channel = guild
            .channels
            .values()
            .map(|channel| channel.read())
            .find(|channel| channel.name == "announcements" && channel.kind == ChannelType::Text);
        if let Some(channel) = channel {
            println!(
                r#"[INFO] Announcing "{}" to discord channel "{}/{}""#,
                data,
                guild.name,
                channel.name()
            );

            let _ = channel.say(&http, data).is_ok(); //Don't let one failure stop the fun
        }
    }
    Ok(())
}
