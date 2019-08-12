use lazy_static::lazy_static;
use rand::seq::SliceRandom;
use serenity::{
    client::Context,
    framework::standard::{
        macros::command,
        Args,
        CommandResult,
    },
    model::channel::Message,
};

const CORPUS: &str = include_str!("moviequotes.memorable_quotes.txt");

lazy_static! {
    static ref QUOTES: Vec<Quote> = {
        let ret: Vec<_> = CORPUS
            .split("\n\n")
            .filter_map(|el| {
                let mut iter = el.trim().lines();
                Some(Quote {
                    movie: iter.next()?,
                    quote: iter.next()?,
                    extra: iter.next()?,
                })
            })
            .collect();
        println!("[INFO] Loaded {} movie quotes", ret.len());
        ret
    };
}

#[derive(Debug)]
struct Quote {
    movie: &'static str,
    quote: &'static str,
    extra: &'static str,
}

#[command]
#[description = "Respond with a random movie quote"]
pub fn movie_quote(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
    let mut rng = rand::thread_rng();
    let quote = match QUOTES.choose(&mut rng) {
        Some(el) => el,
        None => {
            msg.channel_id
                .say(&ctx.http, "Error: Failed to load Quote Corpus")?;
            return Ok(());
        }
    };
    let res = format!("{}\n\t-{}", quote.quote, quote.movie);
    msg.channel_id.say(&ctx.http, &res)?;
    Ok(())
}
