use crate::CommandContext;
use zalgo::ZalgoBuilder;

/// Zalgoify a phrase
#[poise::command(slash_command)]
pub async fn zalgo(
    ctx: CommandContext<'_>,
    #[description = "The text to zalgoify"] text: String,
    #[description = "The length of the output in chars"] max_len: Option<usize>,
) -> anyhow::Result<()> {
    let max_len = max_len.unwrap_or(2000);

    let text_len = text.chars().count();
    let total = (max_len as f32 - text_len as f32) / text_len as f32;
    let max = (total / 3.0) as usize;

    if max == 0 {
        ctx.say("The text cannot be zalgoified within the given limits")
            .await?;
        return Ok(());
    }

    let output = ZalgoBuilder::new()
        .set_up(max)
        .set_down(max)
        .set_mid(max)
        .zalgoify(&text);

    ctx.say(output).await?;

    Ok(())
}
