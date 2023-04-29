use serenity::{
    client::Context,
    framework::standard::{
        macros::command,
        Args,
        CommandResult,
    },
    model::channel::Message,
};

#[command]
#[description("Get an invite link for this bot")]
pub async fn invite(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let app_info = ctx.http.get_current_application_info().await?;

    let id = app_info.id;
    let permissions = "1341644225";
    let link = format!(
        "https://discordapp.com/oauth2/authorize?client_id={id}&scope=bot&permissions={permissions}",
    );
    msg.channel_id.say(&ctx.http, &link).await?;
    Ok(())
}
