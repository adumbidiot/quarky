use crate::{
    CommandContext,
    ReqwestClientKey,
};
use songbird::input::YoutubeDl;

/// Play audio from youtube
#[poise::command(slash_command)]
pub async fn play(
    ctx: CommandContext<'_>,
    #[description = "The url to play from"] url: String,
) -> anyhow::Result<()> {
    let serenity_context = ctx.serenity_context();

    let client = serenity_context
        .data
        .read()
        .await
        .get::<ReqwestClientKey>()
        .unwrap()
        .clone();

    // Validation
    if !url.starts_with("http") {
        ctx.say("Must provide a valid URL").await?;
        return Ok(());
    }

    let maybe_guild_id = ctx.guild_id();
    let guild_id = match maybe_guild_id {
        Some(guild_id) => guild_id,
        None => {
            ctx.say("Error finding channel info").await?;
            return Ok(());
        }
    };

    let manager = songbird::get(serenity_context)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler) = manager.get(guild_id) {
        let mut handler = handler.lock().await;
        let source = YoutubeDl::new(client, url);
        handler.play_only_input(source.into());

        ctx.say("Playing song").await?;
    } else {
        ctx.say("Not in a voice channel to play in").await?;
    }

    Ok(())
}
