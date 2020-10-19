use crate::RedditClientKey;
use indexmap::IndexMap;
use log::{
    info,
    warn,
};
use rand::Rng;
use reddit::{
    Link,
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
    prelude::RwLock,
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

type SubRedditCache = Arc<RwLock<HashMap<String, EntryCache>>>;

#[derive(Default, Clone)]
struct EntryCache {
    store: Arc<RwLock<IndexMap<String, Arc<Link>>>>,
    random_count: Arc<AtomicUsize>,
}

impl EntryCache {
    async fn populate(&self, iter: impl Iterator<Item = Box<Link>>) -> usize {
        let mut map = self.store.write().await;
        let mut added = 0;
        for link in iter {
            if map.insert(link.id.clone(), Arc::from(link)).is_none() {
                added += 1;
            }
        }
        self.random_count.store(0, Ordering::SeqCst);
        added
    }

    async fn get_random(&self) -> Option<Arc<Link>> {
        self.random_count.fetch_add(1, Ordering::SeqCst);

        let store = self.store.read().await;
        let store_len = store.len();

        if store_len == 0 {
            return None;
        }

        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0, store_len);

        store.get_index(index).map(|(_, v)| v).cloned()
    }

    async fn needs_data(&self) -> bool {
        let num = self.store.read().await.len();
        num == 0 || self.random_count.load(Ordering::SeqCst) > (num / 2) // TODO: FineTune / make configurable
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

    async fn populate_cache(&self, subreddit: &str) -> Result<EntryCache, RedditError> {
        let map_arc = self
            .cache
            .write()
            .await
            .entry(String::from(subreddit))
            .or_default()
            .clone();

        let list = self.client.get_subreddit(&subreddit, 100).await?;
        if let Some(listing) = list.data.into_listing() {
            let posts = listing
                .children
                .into_iter()
                .filter_map(|child| child.data.into_link())
                .filter(|link| {
                    link.post_hint == Some(PostHint::Image)
                        || link.url.as_str().ends_with(".jpg")
                        || link.url.as_str().ends_with(".png")
                        || link.url.as_str().ends_with(".gif")
                });

            let new_posts = map_arc.populate(posts).await;
            info!("Reddit Cache populated with {} new posts", new_posts);
        } else {
            warn!("Missing listing for subreddit '{}'", subreddit);
        }

        Ok(map_arc)
    }

    pub async fn get_random_post(&self, subreddit: &str) -> Result<Option<Arc<Link>>, RedditError> {
        let entry_cache = self
            .cache
            .write()
            .await
            .entry(String::from(subreddit))
            .or_default()
            .clone();

        let entry_cache = if entry_cache.needs_data().await {
            self.populate_cache(subreddit).await?
        } else {
            entry_cache
        };

        Ok(entry_cache.get_random().await)
    }
}

#[command]
#[description("Get a random post from a subreddit")]
#[bucket("simple")]
#[min_args(1)]
async fn reddit(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let subreddit = args.single::<String>().unwrap();

    let blacklist = ["gayporn"];
    if blacklist.contains(&subreddit.as_str()) {
        msg.channel_id
            .say(&ctx.http, "*Angry Barking Noises*")
            .await?;
        return Ok(());
    }

    let data_lock = ctx.data.read().await;
    let client = data_lock.get::<RedditClientKey>().unwrap();

    match client.get_random_post(&subreddit).await {
        Ok(Some(post)) => {
            msg.channel_id.say(&ctx.http, &post.url).await?;
        }
        Ok(None) => {
            msg.channel_id
                .say(&ctx.http, "Error: No Image Posts found")
                .await?;
        }
        Err(e) => {
            msg.channel_id
                .say(&ctx.http, format!("Failed fetching posts: {}", e))
                .await?;
        }
    }

    Ok(())
}
