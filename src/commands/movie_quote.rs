use lazy_static::lazy_static;
use rand::{
    seq::SliceRandom,
    Rng,
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

// Note: The Corpus is untouched aside from me removing non-utf8 bytes and converting the line endings to LF
const MEMORABLE_QUOTES_CORPUS: &str =
    include_str!("cornell_movie_quotes_corpus/moviequotes.memorable_quotes.txt");
const QUOTE_PAIRS_CORPUS: &str =
    include_str!("cornell_movie_quotes_corpus/moviequotes.memorable_nonmemorable_pairs.txt");

lazy_static! {
    static ref MEMORABLE_QUOTES: Vec<MemorableQuote> = {
        let ret: Vec<_> = MEMORABLE_QUOTES_CORPUS
            .split("\n\n")
            .filter_map(|el| {
                let mut iter = el.trim().lines();
                Some(MemorableQuote {
                    movie: iter.next()?,
                    quote: iter.next()?,
                    extra: IdQuote::from_line(iter.next()?)?,
                })
            })
            .collect();
        println!("[INFO] Loaded {} Memorable movie quotes", ret.len());
        ret
    };
    static ref QUOTE_PAIRS: Vec<Quote> = {
        let ret: Vec<_> = QUOTE_PAIRS_CORPUS
            .split("\n\n")
            .filter_map(|el| {
                let mut iter = el.trim().lines();
                let movie = iter.next()?;
                let quote = iter.next()?;
                let memorable_quote = IdQuote::from_line(iter.next()?)?;
                let nonmemorable_quote = IdQuote::from_line(iter.next()?)?;

                Some(Quote {
                    movie,
                    quote,
                    memorable_quote,
                    nonmemorable_quote,
                })
            })
            .collect();
        println!("[INFO] Loaded {} movie quote pairs", ret.len());
        ret
    };
}

#[command]
#[description = "Respond with a random movie quote"]
pub fn movie_quote(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
    let mut rng = rand::thread_rng();

    match rng.gen_range(0, 2) {
        0 => {
            let quote = match MEMORABLE_QUOTES.choose(&mut rng) {
                Some(el) => el,
                None => {
                    msg.channel_id
                        .say(&ctx.http, "Error: Failed to load Memorable Quote Corpus")?;
                    return Ok(());
                }
            };
            let res = format!("{}\n\t-{}", quote.quote, quote.movie);
            msg.channel_id.say(&ctx.http, &res)?;
        }
        1 => match QUOTE_PAIRS.choose(&mut rng) {
            Some(quote) => {
                let quote_data = if rng.gen() {
                    quote.memorable_quote.quote
                } else {
                    quote.nonmemorable_quote.quote
                };
                let res = format!("{}\n\t-{}", quote_data, quote.movie);
                msg.channel_id.say(&ctx.http, &res)?;
            }
            None => {
                msg.channel_id
                    .say(&ctx.http, "Error: Failed to load Quote Pair Corpus")?;
                return Ok(());
            }
        },
        _ => unreachable!("Invalid num generated!"),
    }

    Ok(())
}

#[derive(Debug)]
struct MemorableQuote {
    movie: &'static str,
    quote: &'static str,
    extra: IdQuote,
}

#[derive(Debug)]
struct IdQuote {
    id: u64,
    quote: &'static str,
}

impl IdQuote {
    fn from_line(data: &'static str) -> Option<Self> {
        let mut iter = data.splitn(2, ' ');
        let id = iter.next()?.parse().ok()?;
        let quote = iter.next()?;

        Some(IdQuote { id, quote })
    }
}

#[derive(Debug)]
struct Quote {
    movie: &'static str,
    quote: &'static str,
    memorable_quote: IdQuote,
    nonmemorable_quote: IdQuote,
}
