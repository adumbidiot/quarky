use crate::VoiceManagerKey;
use serenity::{
    client::Context,
    framework::standard::{
        macros::command,
        Args,
        CommandResult,
    },
    model::channel::Message,
    voice,
};

#[command]
#[bucket("voice")]
fn play(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            msg.channel_id
                .say(&ctx.http, "Must provide a URL to a video or audio")?;
            return Ok(());
        }
    };

    //Validation
    if !url.starts_with("http") {
        msg.channel_id.say(&ctx.http, "Must provide a valid URL")?;
        return Ok(());
    }

    let guild_id = match ctx.cache.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            msg.channel_id
                .say(&ctx.http, "Error finding channel info")?;
            return Ok(());
        }
    };

    let manager_lock = ctx.data.read().get::<VoiceManagerKey>().cloned().unwrap();
    let mut manager = manager_lock.lock();

    if let Some(handler) = manager.get_mut(guild_id) {
        let source = match voice::ytdl(&url) {
            Ok(source) => source,
            Err(why) => {
                println!("[ERROR] Could not play audio: {:?}", why);
                msg.channel_id.say(&ctx.http, "Error sourcing ffmpeg")?;
                return Ok(());
            }
        };

        handler.play_only(source);

        msg.channel_id.say(&ctx.http, "Playing song")?;
    } else {
        msg.channel_id
            .say(&ctx.http, "Not in a voice channel to play in")?;
    }

    Ok(())
}
