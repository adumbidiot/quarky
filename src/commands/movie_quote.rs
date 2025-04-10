use crate::CommandContext;
use anyhow::Context;
use lazy_static::lazy_static;
use log::info;
use rand::{
    prelude::IndexedRandom,
    Rng,
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
        info!("Loaded {} Memorable movie quotes", ret.len());
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
        info!("Loaded {} movie quote pairs", ret.len());
        ret
    };
}

/// Respond with a random movie quote
#[poise::command(slash_command)]
pub async fn movie_quote(ctx: CommandContext<'_>) -> anyhow::Result<()> {
    let corpus_choice;
    let memorable_quote;
    let quote_pair;
    let use_memorable;
    {
        let mut rng = rand::rng();

        corpus_choice = rng.random_range(0u8..2);

        memorable_quote = MEMORABLE_QUOTES.choose(&mut rng);
        quote_pair = QUOTE_PAIRS.choose(&mut rng);

        use_memorable = rng.random();
    }

    match corpus_choice {
        0 => {
            let quote = match memorable_quote.context("failed to load Memorable Quote Corpus") {
                Ok(el) => el,
                Err(error) => {
                    ctx.say(format!("{error:?}")).await?;
                    return Ok(());
                }
            };
            let response = format!("{}\n\t-{}", quote.quote, quote.movie);
            ctx.say(&response).await?;
        }
        1 => {
            let quote = match quote_pair.context("failed to load Quote Pair Corpus") {
                Ok(quote) => quote,
                Err(error) => {
                    ctx.say(format!("{error:?}")).await?;
                    return Ok(());
                }
            };

            let quote_data = if use_memorable {
                quote.memorable_quote.quote
            } else {
                quote.nonmemorable_quote.quote
            };
            let response = format!("{}\n\t-{}", quote_data, quote.movie);
            ctx.say(&response).await?;
        }
        _ => unreachable!("Invalid num generated!"),
    }

    Ok(())
}

#[derive(Debug)]
pub struct MemorableQuote {
    pub movie: &'static str,
    pub quote: &'static str,
    #[allow(dead_code)]
    pub extra: IdQuote,
}

#[derive(Debug)]
pub struct IdQuote {
    #[allow(dead_code)]
    pub id: u64,
    #[allow(dead_code)]
    pub quote: &'static str,
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
pub struct Quote {
    pub movie: &'static str,
    #[allow(dead_code)]
    pub quote: &'static str,
    pub memorable_quote: IdQuote,
    pub nonmemorable_quote: IdQuote,
}
