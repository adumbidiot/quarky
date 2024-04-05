#![allow(deprecated)]

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
    framework::standard::{
        buckets::BucketBuilder,
        help_commands,
        macros::{
            group,
            help,
        },
        Args,
        CommandGroup,
        CommandResult,
        Configuration as StandardFrameworkConfiguration,
        DispatchError,
        HelpOptions,
        StandardFramework,
    },
    gateway::ActivityData,
    model::{
        channel::{
            Channel,
            Message,
        },
        gateway::Ready,
        id::UserId,
        voice::VoiceState,
    },
    prelude::*,
};
use songbird::SerenityInit;
use std::{
    collections::HashSet,
    sync::Arc,
    time::Duration,
};
use tokio::sync::Notify;

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
    if let Err(error) = help_commands::with_embeds(context, msg, args, help_options, groups, owners)
        .await
        .context("failed to send help")
    {
        error!("{error:?}");
    }
    Ok(())
}

struct RedditClientKey;

impl TypeMapKey for RedditClientKey {
    type Value = RedditClient;
}

struct RssClientKey;

impl TypeMapKey for RssClientKey {
    type Value = rss_client::Client;
}

struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        let random_number: u8 = rand::thread_rng().gen_range(0..2);
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
    let rss_client = client
        .data
        .read()
        .await
        .get::<RssClientKey>()
        .cloned()
        .unwrap();

    scheduler.every(day).at(time).run(move || {
        let msg = msg.clone();
        let http = http.clone();
        let cache = cache.clone();
        let rss_client = rss_client.clone();
        tokio::spawn(async move {
            let msg = match self::random_tweet::get_random_tweet_url(&rss_client, "dog_rates").await
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
    info!("Using prefix \"{}\"", config.prefix);

    // Init Framework
    let standard_framework_configuration =
        StandardFrameworkConfiguration::new().prefix(&config.prefix);

    let framework = StandardFramework::new();
    framework.configure(standard_framework_configuration);
    let framework = framework
        .group(&GENERAL_GROUP)
        .help(&HELP)
        .on_dispatch_error(|ctx, msg, error, cmd_name| {
            Box::pin(async move {
                match error {
                    DispatchError::Ratelimited(duration) => {
                        if let Err(error) = msg
                            .channel_id
                            .say(
                                &ctx.http,
                                format!(
                                    "Try this again in {} second(s).",
                                    duration.rate_limit.as_secs_f32()
                                ),
                            )
                            .await
                        {
                            warn!("Failed to send ratelimited warning message: {error}");
                        }
                    }
                    DispatchError::CommandDisabled => {
                        if let Err(error) = msg
                            .channel_id
                            .say(&ctx.http, format!("Command \"{cmd_name}\" disabled."))
                            .await
                        {
                            warn!("Failed to send disabled command warning message: {error}");
                        }
                    }
                    DispatchError::NotEnoughArguments { min, given } => {
                        if let Err(error) = msg
                            .channel_id
                            .say(
                                &ctx.http,
                                format!("Need {min} arguments but only got {given}"),
                            )
                            .await
                        {
                            warn!("Failed to send not enough args warning message: {error}");
                        }
                    }
                    DispatchError::TooManyArguments { max, given } => {
                        if let Err(error) = msg
                            .channel_id
                            .say(
                                &ctx.http,
                                format!("Need only {max} arguments but got {given}"),
                            )
                            .await
                        {
                            warn!("Failed to send too many args warning message: {error}");
                        }
                    }
                    _ => {
                        warn!("DispatchError: {:?} | {}", error, msg.content);
                    }
                }
            })
        })
        .bucket("simple", BucketBuilder::default().delay(1))
        .await
        .bucket("voice", BucketBuilder::default().delay(1))
        .await;

    let mut client = Client::builder(
        &config.token,
        GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT,
    )
    .event_handler(Handler)
    .framework(framework)
    .register_songbird()
    .await
    .context("failed to build client")?;

    let reddit_client = RedditClient::new();
    let rss_client = rss_client::Client::new();
    {
        let mut client_data = client.data.write().await;

        client_data.insert::<RedditClientKey>(reddit_client);
        client_data.insert::<RssClientKey>(rss_client);
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
