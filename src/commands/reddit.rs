use crate::{
    RedditClientKey,
    TokioRuntimeKey,
};
use parking_lot::RwLock;
use rand::seq::IteratorRandom;
use reddit::{
    PostHint,
    RedditError,
    SubRedditEntry,
    SubRedditEntryData,
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
use std::{
    collections::HashMap,
    sync::{
        atomic::{
            AtomicUsize,
            Ordering,
        },
        Arc,
    },
};
use tokio::prelude::{
    future::Either,
    Future,
    IntoFuture,
};

type SubRedditCache = Arc<RwLock<HashMap<String, EntryCache>>>;

#[derive(Default, Clone)]
struct EntryCache {
    store: Arc<RwLock<HashMap<String, Arc<SubRedditEntryData>>>>,
    random_count: Arc<AtomicUsize>,
}

impl EntryCache {
    fn populate(&self, iter: impl Iterator<Item = SubRedditEntry>) -> usize {
        let mut map = self.store.write();
        let mut added = 0;
        for post in iter {
            if map
                .insert(post.data.id.clone(), Arc::from(post.data))
                .is_none()
            {
                added += 1;
            }
        }
        self.random_count.store(0, Ordering::SeqCst);
        added
    }

    fn get_random(&self) -> Option<Arc<SubRedditEntryData>> {
        self.random_count.fetch_add(1, Ordering::SeqCst);
        let mut rng = rand::thread_rng();
        self.store
            .read()
            .iter()
            .map(|(_, v)| v)
            .choose(&mut rng)
            .cloned()
    }

    fn needs_data(&self) -> bool {
        let num = self.store.read().len();
        num == 0 || self.random_count.load(Ordering::SeqCst) > (num / 2) //TODO: FineTune / make configurable
    }
}

pub struct RedditClient {
    client: reddit::Client,
    cache: SubRedditCache,
}

impl RedditClient {
    pub fn new() -> Self {
        RedditClient {
            client: reddit::Client::new(),
            cache: Default::default(),
        }
    }

    fn populate_cache(
        &self,
        subreddit: &str,
    ) -> impl Future<Item = EntryCache, Error = RedditError> {
        let map_arc = self
            .cache
            .write()
            .entry(String::from(subreddit))
            .or_default()
            .clone();

        self.client.get_subreddit(&subreddit, 100).map(move |list| {
            let posts = list.data.children.into_iter().filter(|child| {
                child.data.post_hint == Some(PostHint::Image)
                    || child.data.url.ends_with(".jpg")
                    || child.data.url.ends_with(".png")
                    || child.data.url.ends_with(".gif")
            });

            let new_posts = map_arc.populate(posts);
            println!("[INFO] Reddit Cache populated with {} new posts", new_posts);

            map_arc
        })
    }

    pub fn get_random_post(
        &self,
        subreddit: &str,
    ) -> impl Future<Item = Option<Arc<SubRedditEntryData>>, Error = RedditError> {
        let entry_cache = self
            .cache
            .write()
            .entry(String::from(subreddit))
            .or_default()
            .clone();

        let fut = if entry_cache.needs_data() {
            Either::A(self.populate_cache(subreddit))
        } else {
            Either::B(Ok(entry_cache).into_future())
        };

        fut.map(|entry_cache| entry_cache.get_random())
    }
}

#[command]
#[description = "Get a random post from a subreddit"]
#[min_args(1)]
fn reddit(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let subreddit = args.single::<String>().unwrap();

    let data_lock = ctx.data.read();
    let rt = data_lock.get::<TokioRuntimeKey>().unwrap();
    let client = data_lock.get::<RedditClientKey>().unwrap();

    match rt.write().block_on(client.get_random_post(&subreddit)) {
        Ok(Some(post)) => {
            let _ = msg.channel_id.say(&ctx.http, &post.url)?;
        }
        Ok(None) => {
            let _ = msg
                .channel_id
                .say(&ctx.http, "Error: No Image Posts found")?;
        }
        Err(e) => match e {
            RedditError::NotFound => {
                let _ = msg.channel_id.say(&ctx.http, "Subreddit not found")?;
            }
            RedditError::Json(e, _buffer) => {
                let _ = msg
                    .channel_id
                    .say(&ctx.http, &format!("Json Error: {:#?}", e))?;
            }
            _ => {
                let _ = msg.channel_id.say(&ctx.http, &format!("Error: {:#?}", e))?;
            }
        },
    }

    Ok(())
}
