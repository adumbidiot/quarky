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
    prelude::*,
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
    stop,
    random_tweet
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
    help_commands::with_embeds(context, msg, args, &help_options, groups, owners).await;
    Ok(())
}

struct RedditClientKey;

impl TypeMapKey for RedditClientKey {
    type Value = Arc<RedditClient>;
}

struct VoiceManagerKey;

impl TypeMapKey for VoiceManagerKey {
    type Value = Arc<Mutex<ClientVoiceManager>>;
}

pub struct TwitterTokenKey;

impl TypeMapKey for TwitterTokenKey {
    type Value = Arc<egg_mode::auth::Token>;
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

async fn schedule_robotics_reminder(
    client: &Client,
    scheduler: &mut Scheduler,
    day: clokwerk::Interval,
    time: &str,
    msg: &str,
) {
    let data_lock = client.data.read().await;
    let token = data_lock.get::<TwitterTokenKey>().unwrap().clone();
    drop(data_lock);

    let msg = msg.to_string();
    let http = client.cache_and_http.http.clone();
    let cache = client.cache_and_http.cache.clone();

    scheduler.every(day).at(time).run(move || {
        let token = token.clone();
        let msg = msg.clone();
        let http = http.clone();
        let cache = cache.clone();
        tokio::spawn(async move {
            let msg = match crate::random_tweet::get_random_tweet_url(&token, "dog_rates")
                .await
                .ok()
                .flatten()
            {
                Some(link) => format!("{}\n{}", msg, link),
                None => msg,
            };

            // TODO: Ensure client is started and connected before running
            let _ = announce_discord(&http, &cache, &msg).await.is_ok();
        });
    });
}

fn main() {
    println!("[INFO] Loading Config.toml...");
    let config_path = "./Config.toml";
    let config = match load_config(Path::new(config_path)) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("[ERROR] Error loading '{}': {:#?}", config_path, e);
            return;
        }
    };

    let mut tokio_runtime = match TokioRuntime::new() {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("[ERROR] Error starting tokio runtime: {}", e);
            return;
        }
    };

    let twitter_token = egg_mode::auth::Token::Bearer(config.twitter.bearer_token.clone());
    match tokio_runtime.block_on(egg_mode::auth::verify_tokens(&twitter_token)) {
        Ok(user) => {
            println!(
                "[INFO] Using twitter api from '{}({})'",
                user.screen_name, user.id
            );
        }
        Err(e) => {
            // This might only be for api key/secret? warn only for now
            eprintln!("[WARN] Invalid Twitter Token: {}", e);
        }
    }

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
                                format!("Try this again in {} second(s).", seconds.as_secs_f32()),
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
                eprintln!("Failed to start client: {}", e);
                return;
            }
        };

        let reddit_client = Arc::from(RedditClient::new());

        {
            let mut client_data = client.data.write().await;

            client_data.insert::<RedditClientKey>(reddit_client);
            client_data.insert::<VoiceManagerKey>(Arc::clone(&client.voice_manager));
            client_data.insert::<TwitterTokenKey>(Arc::new(twitter_token));
        }

        // Start Scheduler
        println!("[INFO] Starting Event Scheduler...");
        // TODO: Wrap in arc and rwlock for dynamically adding and removing events?
        let mut scheduler = Scheduler::new();
        const AFTER_SCHOOL_ANNOUNCE: &str = "@everyone Robotics Club after school today!";
        const LUNCH_ANNOUNCE: &str = "@everyone Robotics Club during plus and lunch today!";
        const NOON: &str = "12:00:00";

        schedule_robotics_reminder(&client, &mut scheduler, Monday, NOON, AFTER_SCHOOL_ANNOUNCE)
            .await;
        schedule_robotics_reminder(&client, &mut scheduler, Tuesday, NOON, LUNCH_ANNOUNCE).await;
        schedule_robotics_reminder(&client, &mut scheduler, Thursday, NOON, LUNCH_ANNOUNCE).await;
        schedule_robotics_reminder(&client, &mut scheduler, Friday, NOON, AFTER_SCHOOL_ANNOUNCE)
            .await;

        let frequency = Duration::from_secs(60 * 5);
        let stop = Arc::new(AtomicBool::new(false));
        let my_stop = stop.clone();
        let handle = tokio::task::spawn(async move {
            while !stop.load(Ordering::SeqCst) {
                scheduler.run_pending();
                tokio::time::delay_for(frequency).await;
            }
        });

        {
            let shard_manager = client.shard_manager.clone();
            tokio::spawn(async move {
                match tokio::signal::ctrl_c().await {
                    Ok(()) => {
                        println!("[INFO] Beginning shutdown...");
                        shard_manager.lock().await.shutdown_all().await;
                    }
                    Err(e) => {
                        eprintln!("[WARN] Failed to register ctrl-c handler: {}", e);
                    }
                }
            });
        }

        println!("[INFO] Logging in...");
        if let Err(why) = client.start().await {
            println!("[ERROR] {}", why);
        }

        println!("[INFO] Shutting down...");
        my_stop.store(true, Ordering::SeqCst);
        drop(client); // Hopefully gets rid of all other Arcs...

        handle.await.unwrap(); // TODO: Actually manage the task better
    });

    drop(tokio_runtime);
}
