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
    ANNOUNCE_COMMAND,
    MOVIE_QUOTE_COMMAND,
    PING_COMMAND,
    REDDIT_COMMAND,
    ZALGO_COMMAND,
};
use config::load_config;
use parking_lot::RwLock;
use rand::Rng;
use serenity::{
    client::{
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
        channel::Message,
        gateway::{
            Activity,
            Ready,
        },
        id::UserId,
    },
    prelude::TypeMapKey,
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
    thread,
    time::Duration,
};
use tokio::prelude::Future;

group!({
    name: "general",
    options: {},
    commands: [ping, announce, reddit, movie_quote, zalgo]
});

#[help]
fn help(
    context: &mut Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, &help_options, groups, owners)
}

struct TokioRuntimeKey;

impl TypeMapKey for TokioRuntimeKey {
    type Value = Arc<RwLock<tokio::runtime::Runtime>>;
}

struct RedditClientKey;

impl TypeMapKey for RedditClientKey {
    type Value = Arc<RedditClient>;
}

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, ctx: Context, ready: Ready) {
        let random_number: u8 = rand::thread_rng().gen_range(0, 2);
        match random_number {
            0 => {
                ctx.set_activity(Activity::playing("with my tail"));
            }
            _ => {
                ctx.set_activity(Activity::listening("hooman noises"));
            }
        }

        println!("[INFO] Choosing Game State {}", random_number);
        println!("[INFO] {} is connected!", ready.user.name);

        //Wait for cache to populate...
        std::thread::sleep(Duration::from_millis(1000));
        //Things that need the cache...
    }

    fn message(&self, _ctx: Context, _msg: Message) {}
}

fn main() {
    println!("[INFO] Loading Config.toml...");
    let config_path = "./Config.toml";
    let config = match load_config(Path::new(config_path)) {
        Ok(config) => config,
        Err(e) => {
            println!("[ERROR] Error loading '{}': {:?}", config_path, e);
            return;
        }
    };

    let mut client = Client::new(&config.token, Handler).expect("Error creating client");

    //Init Framework
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~"))
        .group(&GENERAL_GROUP)
        .help(&HELP)
        .on_dispatch_error(|ctx, msg, error| {
            if let DispatchError::Ratelimited(seconds) = error {
                let _ = msg.channel_id.say(
                    &ctx.http,
                    &format!("Try this again in {} second(s).", seconds),
                );
            } else {
                println!("[ERROR] {:?} {}", error, msg.content);
            }
        })
        .bucket("simple", |b| b.delay(1));

    client.with_framework(framework);

    let tokio_runtime = Arc::from(RwLock::from(
        tokio::runtime::Runtime::new().expect("Error initializing tokio runtime"),
    ));
    client
        .data
        .write()
        .insert::<TokioRuntimeKey>(tokio_runtime.clone());

    let reddit_client = Arc::from(RedditClient::new());
    client.data.write().insert::<RedditClientKey>(reddit_client);

    //Start Scheduler
    let http = client.cache_and_http.http.clone();
    let cache = client.cache_and_http.cache.clone();

    println!("[INFO] Starting Event Scheduler...");
    //TODO: Wrap in arc and rwlock for dynamically adding and removing events?
    let mut scheduler = Scheduler::new();
    const AFTER_SCHOOL_ANNOUNCE: &str = "@everyone Robotics Club after school today!";
    const LUNCH_ANNOUNCE: &str = "@everyone Robotics Club during plus and lunch today!";
    const NOON: &str = "12:00:00";
    {
        let http = http.clone();
        let cache = cache.clone();
        scheduler.every(Monday).at(NOON).run(move || {
            //TODO: Ensure client is started and connected before running
            let _ = announce_discord(&http, &cache.read(), AFTER_SCHOOL_ANNOUNCE).is_ok();
        });
    }
    {
        let http = http.clone();
        let cache = cache.clone();
        scheduler.every(Tuesday).at(NOON).run(move || {
            let _ = announce_discord(&http, &cache.read(), LUNCH_ANNOUNCE).is_ok();
        });
    }
    {
        let http = http.clone();
        let cache = cache.clone();
        scheduler.every(Thursday).at(NOON).run(move || {
            let _ = announce_discord(&http, &cache.read(), LUNCH_ANNOUNCE).is_ok();
        });
    }
    {
        let http = http.clone();
        let cache = cache.clone();
        scheduler.every(Friday).at(NOON).run(move || {
            let _ = announce_discord(&http, &cache.read(), AFTER_SCHOOL_ANNOUNCE).is_ok();
        });
    }

    let frequency = Duration::from_secs(60 * 5);
    let stop = Arc::new(AtomicBool::new(false));
    let my_stop = stop.clone();
    let handle = thread::spawn(move || {
        while !stop.load(Ordering::SeqCst) {
            scheduler.run_pending();
            thread::sleep(frequency);
        }
    });

    println!("[INFO] Logging in...");
    if let Err(why) = client.start() {
        println!("[ERROR] {:?}", why);
    }

    println!("[INFO] Shutting down...");
    drop(client); //Hopefully gets rid of all other Arcs...
    Arc::try_unwrap(tokio_runtime) //TODO: Maybe make a wrapper so this isn't so easy to mess up
        .expect("Should only be one arc left at this point")
        .into_inner()
        .shutdown_on_idle()
        .wait()
        .expect("Tokio Runtime could not exit safely");

    my_stop.store(true, Ordering::SeqCst);
    handle.join().unwrap(); //TODO: Actually manage the thread better
}
