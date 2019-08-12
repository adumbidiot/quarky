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
        CORPUS
            .split("\r\n\r\n")
            .filter_map(|el| {
                let mut iter = el.trim().split("\r\n");
                Some(Quote {
                    movie: iter.next()?,
                    quote: iter.next()?,
                    extra: iter.next()?,
                })
            })
            .collect()
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
    let quote = QUOTES.choose(&mut rng).expect("Quote");
    msg.channel_id
        .say(&ctx.http, &format!("{}\n\t~ {}", quote.quote, quote.movie))?;
    Ok(())
}
