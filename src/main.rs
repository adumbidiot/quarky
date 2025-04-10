mod cli_options;
mod commands;
mod config;
mod logger;

use self::{
    cli_options::CliOptions,
    commands::{
        announce::announce_discord,
        reddit::RedditClient,
        *,
    },
    config::Config,
};
use anyhow::Context as _;
use clokwerk::{
    Interval::{
        Friday,
        Monday,
        Thursday,
        Tuesday,
    },
    Job,
    Scheduler,
};
use log::{
    error,
    info,
    warn,
};
use rand::Rng;
use serenity::{
    gateway::ActivityData,
    model::{
        channel::{
            Channel,
            Message,
        },
        gateway::Ready,
        voice::VoiceState,
    },
    prelude::*,
};
use songbird::SerenityInit;
use std::{
    sync::Arc,
    time::Duration,
};
use tokio::sync::Notify;

pub type CommandContext<'a> = poise::Context<'a, (), anyhow::Error>;

struct RedditClientKey;

impl TypeMapKey for RedditClientKey {
    type Value = Arc<RedditClient>;
}

struct RssClientKey;

impl TypeMapKey for RssClientKey {
    type Value = rss_client::Client;
}

struct NitterClientKey;

impl TypeMapKey for NitterClientKey {
    type Value = nitter::Client;
}

struct ReqwestClientKey;

impl TypeMapKey for ReqwestClientKey {
    type Value = reqwest::Client;
}

struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        let random_number: u8 = rand::rng().random_range(0..2);
        match random_number {
            0 => {
                ctx.set_activity(Some(ActivityData::playing("with my tail")));
            }
            _ => {
                ctx.set_activity(Some(ActivityData::listening("hooman noises")));
            }
        }

        info!("choosing game state {random_number}");
        info!("{} is connected!", ready.user.name);
    }

    async fn message(&self, _ctx: Context, _msg: Message) {}

    async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        #[allow(clippy::collapsible_match)]
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
                            if let Ok(members) = channel.members(&ctx.cache) {
                                if members.len() == 1
                                    && ctx
                                        .http
                                        .get_current_user()
                                        .await
                                        .map(|u| u.id == members[0].user.id)
                                        .unwrap_or(false)
                                {
                                    let manager = songbird::get(&ctx)
                                        .await
                                        .expect("missing songbird data")
                                        .clone();
                                    let has_handler = manager.get(channel.guild_id).is_some();
                                    if has_handler {
                                        if let Err(e) = manager.leave(channel.guild_id).await {
                                            warn!("failed to leave voice channel: {}", e);
                                        }

                                        if let Err(e) = manager.remove(channel.guild_id).await {
                                            warn!("failed to remove voice channel: {}", e);
                                        }
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
    let msg = msg.to_string();
    let http = client.http.clone();
    let cache = client.cache.clone();
    let rss_client;
    let nitter_client;
    {
        let data = client.data.read().await;

        rss_client = data.get::<RssClientKey>().cloned().unwrap();
        nitter_client = data.get::<NitterClientKey>().cloned().unwrap();
    }

    scheduler.every(day).at(time).run(move || {
        let msg = msg.clone();
        let http = http.clone();
        let cache = cache.clone();
        let rss_client = rss_client.clone();
        let nitter_client = nitter_client.clone();
        tokio::spawn(async move {
            let msg = match self::random_tweet::get_random_tweet_url(
                &rss_client,
                &nitter_client,
                "dog_rates",
            )
            .await
            {
                Ok(Some(link)) => format!("{msg}\n{link}"),
                Ok(None) => {
                    error!("feed empty");
                    msg
                }
                Err(error) => {
                    error!("{error:?}");
                    msg
                }
            };

            // TODO: Ensure client is started and connected before running
            announce_discord(&http, &cache, &msg).await;
        });
    });
}

fn main() -> anyhow::Result<()> {
    let cli_options: CliOptions = argh::from_env();
    eprintln!("loading config @ \"{}\"...", cli_options.config);
    let config = Config::load(&cli_options.config)
        .with_context(|| format!("failed to load \"{}\"", &cli_options.config))?;
    self::logger::setup(&config).context("failed to setup logger")?;

    let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    tokio_runtime.block_on(async_main(config))?;
    drop(tokio_runtime);

    Ok(())
}

async fn async_main(config: Config) -> anyhow::Result<()> {
    // Init Framework
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                zalgo(),
                help(),
                vaporwave(),
                stop(),
                reddit(),
                random_tweet(),
                play(),
                ping(),
                movie_quote(),
                leave(),
                join(),
                invite(),
                announce(),
            ],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(())
            })
        })
        .build();

    let mut client = Client::builder(&config.token, GatewayIntents::non_privileged())
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .await
        .context("failed to build client")?;

    let reddit_client = Arc::new(RedditClient::new());
    let rss_client = rss_client::Client::new();
    let nitter_client = nitter::Client::new();
    let reqwest_client = reqwest::Client::new();
    {
        let mut client_data = client.data.write().await;

        client_data.insert::<RedditClientKey>(reddit_client);
        client_data.insert::<RssClientKey>(rss_client);
        client_data.insert::<NitterClientKey>(nitter_client);
        client_data.insert::<ReqwestClientKey>(reqwest_client);
    }

    // Start Scheduler
    info!("Starting Event Scheduler...");
    // TODO: Wrap in arc and rwlock for dynamically adding and removing events?
    let mut scheduler = Scheduler::new();
    const AFTER_SCHOOL_ANNOUNCE: &str = "@everyone Robotics Club after school today!";
    const LUNCH_ANNOUNCE: &str = "@everyone Robotics Club during plus and lunch today!";
    const NOON: &str = "12:00:00";
    // TODO: This is flawed and we are lucky it is working correctly (spurrious wakeups).
    // Replace with a better abstraction.
    let scheduler_shutdown_notify = Arc::new(Notify::new());

    schedule_robotics_reminder(&client, &mut scheduler, Monday, NOON, AFTER_SCHOOL_ANNOUNCE).await;
    schedule_robotics_reminder(&client, &mut scheduler, Tuesday, NOON, LUNCH_ANNOUNCE).await;
    schedule_robotics_reminder(&client, &mut scheduler, Thursday, NOON, LUNCH_ANNOUNCE).await;
    schedule_robotics_reminder(&client, &mut scheduler, Friday, NOON, AFTER_SCHOOL_ANNOUNCE).await;

    let frequency = Duration::from_secs(60 * 5);
    let notify = scheduler_shutdown_notify.clone();
    let handle = tokio::task::spawn(async move {
        let mut should_exit = false;

        while !should_exit {
            should_exit = tokio::select! {
                _ = tokio::time::sleep(frequency) => false,
                _ = notify.notified() => true,

            };

            scheduler.run_pending();
        }
    });

    {
        let shard_manager = client.shard_manager.clone();
        tokio::spawn(async move {
            match tokio::signal::ctrl_c().await {
                Ok(()) => {
                    info!("Beginning shutdown...");
                    shard_manager.shutdown_all().await;
                }
                Err(error) => {
                    warn!("Failed to register ctrl-c handler: {error}");
                }
            }
        });
    }

    info!("Logging in...");
    if let Err(error) = client.start().await {
        error!("Error running client: {error}");
    }

    info!("Shutting down...");
    scheduler_shutdown_notify.notify_one();
    drop(client); // Hopefully gets rid of all other Arcs...

    if let Err(error) = handle.await {
        error!("Scheduler Crashed: {error}");
    }

    Ok(())
}
