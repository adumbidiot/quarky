use crate::{
    RedditClientKey,
    TokioRuntimeKey,
};
use rand::seq::IteratorRandom;
use reddit::{
    PostHint,
    RedditError,
};
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
#[description = "Get a random post from a subreddit"]
#[min_args(1)]
fn reddit(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let subreddit = args.single::<String>().unwrap();

    let data_lock = ctx.data.read();
    let rt = data_lock.get::<TokioRuntimeKey>().unwrap();
    let client = data_lock.get::<RedditClientKey>().unwrap();

    match rt.write().block_on(client.get_subreddit(&subreddit)) {
        Ok(list) => {
            let mut rng = rand::thread_rng();
            let post = list
                .data
                .children
                .iter()
                .filter(|child| child.data.post_hint == PostHint::Image)
                .choose(&mut rng);
            if let Some(post) = post {
                let _ = msg.channel_id.say(&ctx.http, &post.data.url)?;
            } else {
                let _ = msg
                    .channel_id
                    .say(&ctx.http, "Error: No Image Posts found")?;
            }
        }
        Err(e) => match e {
            RedditError::NotFound => {
                let _ = msg.channel_id.say(&ctx.http, "Subreddit not found")?;
            }
            _ => {
                let _ = msg.channel_id.say(&ctx.http, &format!("Error: {:#?}", e))?;
            }
        },
    }

    Ok(())
}
