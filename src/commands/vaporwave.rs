use crate::CommandContext;

pub fn vaporwave_str(data: &str) -> String {
    data.chars()
        .filter_map(|c| {
            let c = c as u32;
            if (33..=270).contains(&c) {
                std::char::from_u32(c + 65248) // unwrap or c ?
            } else {
                Some(32 as char)
            }
        })
        .collect()
}

/// Vaporwave a phrase
#[poise::command(slash_command)]
pub async fn vaporwave(
    ctx: CommandContext<'_>,
    #[description = "The text to vaporwave"] text: String,
) -> anyhow::Result<()> {
    let text = vaporwave_str(&text);
    ctx.say(text).await?;

    Ok(())
}
