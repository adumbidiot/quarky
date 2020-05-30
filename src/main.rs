mod commands;
mod config;

use clokwerk::{
    Interval::{
        Friday,
        Monday,
        Thursday,
        Tuesday,
    },
    Scheduler,
};
use commands::{
    announce::announce_discord,
    reddit::RedditClient,
    *,
};
use config::load_config;
use rand::Rng;
use serenity::{
    client::{
        bridge::voice::ClientVoiceManager,
        Client,
        Context,
        EventHandler,
    },
    framework::standard::{
        help_commands,
        macros::{
            group,
            help,
        },
        Args,
        CommandGroup,
        CommandResult,
        DispatchError,
        HelpOptions,
        StandardFramework,
    },
    model::{
        channel::{
            Channel,
            Message,
        },
        gateway::{
            Activity,
            Ready,
        },
        id::{
            GuildId,
            UserId,
        },
        voice::VoiceState,
    },
    prelude::{
        Mutex,
        TypeMapKey,
    },
};
use std::{
    collections::HashSet,
    path::Path,
    sync::{
        atomic::{
            AtomicBool,
            Ordering,
        },
        Arc,
    },
    time::Duration,
};
use tokio::runtime::Runtime as TokioRuntime;

#[group]
#[commands(
    ping,
    announce,
    reddit,
    movie_quote,
    zalgo,
    vaporwave,
    invite,
    join,
    leave,
    play,
    stop
)]
struct General;

#[help]
async fn help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, &help_options, groups, owners).await
}

struct RedditClientKey;

impl TypeMapKey for RedditClientKey {
    type Value = Arc<RedditClient>;
}

struct VoiceManagerKey;

impl TypeMapKey for VoiceManagerKey {
    type Value = Arc<Mutex<ClientVoiceManager>>;
}

struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        let random_number: u8 = rand::thread_rng().gen_range(0, 2);
        match random_number {
            0 => {
                ctx.set_activity(Activity::playing("with my tail")).await;
            }
            _ => {
                ctx.set_activity(Activity::listening("hooman noises")).await;
            }
        }

        println!("[INFO] Choosing Game State {}", random_number);
        println!("[INFO] {} is connected!", ready.user.name);
    }

    async fn message(&self, _ctx: Context, _msg: Message) {}

    async fn voice_state_update(
        &self,
        ctx: Context,
        _: Option<GuildId>,
        old: Option<VoiceState>,
        new: VoiceState,
    ) {
        if let Some(old_id) = old.and_then(|old| old.channel_id) {
            if new
                .user_id
                .to_user(ctx.http.clone())
                .await
                .map(|user| !user.bot)
                .unwrap_or(false)
            {
                if let Ok(ch) = old_id.to_channel(ctx.http.clone()).await {
                    // I don't think i'm doing this right...
                    #[allow(clippy::single_match)]
                    match ch {
                        Channel::Guild(channel) => {
                            if let Ok(members) = channel.members(ctx.cache).await {
                                if members.len() == 1
                                    && ctx
                                        .http
                                        .get_current_user()
                                        .await
                                        .map(|u| u.id == members[0].user.id)
                                        .unwrap_or(false)
                                {
                                    let manager_lock = ctx
                                        .data
                                        .read()
                                        .await
                                        .get::<VoiceManagerKey>()
                                        .cloned()
                                        .unwrap();
                                    let mut manager = manager_lock.lock().await;
                                    let has_handler = manager.get(channel.guild_id).is_some();
                                    if has_handler {
                                        manager.leave(channel.guild_id);
                                        manager.remove(channel.guild_id);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

fn main() {
    println!("[INFO] Loading Config.toml...");
    let config_path = "./Config.toml";
    let config = match load_config(Path::new(config_path)) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("[ERROR] Error loading '{}': {:?}", config_path, e);
            return;
        }
    };

    let mut tokio_runtime = match TokioRuntime::new() {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("[ERROR] Error starting tokio runtime: {:?}", e);
            return;
        }
    };

    tokio_runtime.block_on(async {
        // Init Framework
        let framework = StandardFramework::new()
            .configure(|c| c.prefix("~"))
            .group(&GENERAL_GROUP)
            .help(&HELP)
            .on_dispatch_error(|ctx, msg, error| {
                if let DispatchError::Ratelimited(seconds) = error {
                    Box::pin(async move {
                        let _ = msg
                            .channel_id
                            .say(
                                &ctx.http,
                                &format!("Try this again in {} second(s).", seconds),
                            )
                            .await;
                    })
                } else {
                    println!("[ERROR] {:?} {}", error, msg.content);
                    Box::pin(async {})
                }
            })
            .bucket("simple", |b| b.delay(1))
            .await
            .bucket("voice", |b| b.delay(1))
            .await;

        let mut client = match Client::new(&config.token)
            .event_handler(Handler)
            .framework(framework)
            .await
        {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to start client: {:#?}", e);
                return;
            }
        };

        let reddit_client = Arc::from(RedditClient::new());

        {
            let mut client_data = client.data.write().await;

            client_data.insert::<RedditClientKey>(reddit_client);
            client_data.insert::<VoiceManagerKey>(Arc::clone(&client.voice_manager));
        }

        // Start Scheduler
        let http = client.cache_and_http.http.clone();
        let cache = client.cache_and_http.cache.clone();

        println!("[INFO] Starting Event Scheduler...");
        // TODO: Wrap in arc and rwlock for dynamically adding and removing events?
        let mut scheduler = Scheduler::new();
        const AFTER_SCHOOL_ANNOUNCE: &str = "@everyone Robotics Club after school today!";
        const LUNCH_ANNOUNCE: &str = "@everyone Robotics Club during plus and lunch today!";
        const NOON: &str = "12:00:00";
        {
            let http = http.clone();
            let cache = cache.clone();

            scheduler.every(Monday).at(NOON).run(move || {
                let http = http.clone();
                let cache = cache.clone();
                tokio::spawn(async move {
                    // TODO: Ensure client is started and connected before running
                    let _ = announce_discord(&http, &cache, AFTER_SCHOOL_ANNOUNCE)
                        .await
                        .is_ok();
                });
            });
        }
        {
            let http = http.clone();
            let cache = cache.clone();
            scheduler.every(Tuesday).at(NOON).run(move || {
                let http = http.clone();
                let cache = cache.clone();
                tokio::spawn(async move {
                    let _ = announce_discord(&http, &cache, LUNCH_ANNOUNCE)
                        .await
                        .is_ok();
                });
            });
        }
        {
            let http = http.clone();
            let cache = cache.clone();
            scheduler.every(Thursday).at(NOON).run(move || {
                let http = http.clone();
                let cache = cache.clone();
                tokio::spawn(async move {
                    let _ = announce_discord(&http, &cache, LUNCH_ANNOUNCE)
                        .await
                        .is_ok();
                });
            });
        }
        {
            let http = http;
            let cache = cache;
            scheduler.every(Friday).at(NOON).run(move || {
                let http = http.clone();
                let cache = cache.clone();
                tokio::spawn(async move {
                    let _ = announce_discord(&http, &cache, AFTER_SCHOOL_ANNOUNCE)
                        .await
                        .is_ok();
                });
            });
        }

        let frequency = Duration::from_secs(60 * 5);
        let stop = Arc::new(AtomicBool::new(false));
        let my_stop = stop.clone();
        let handle = tokio::task::spawn(async move {
            while !stop.load(Ordering::SeqCst) {
                scheduler.run_pending();
                tokio::time::delay_for(frequency).await;
            }
        });

        println!("[INFO] Logging in...");
        if let Err(why) = client.start().await {
            println!("[ERROR] {:?}", why);
        }

        println!("[INFO] Shutting down...");
        my_stop.store(true, Ordering::SeqCst);
        drop(client); // Hopefully gets rid of all other Arcs...

        handle.await.unwrap(); // TODO: Actually manage the task better
    });

    drop(tokio_runtime);
}
