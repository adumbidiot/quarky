mod commands;

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
    ANNOUNCE_COMMAND,
    PING_COMMAND,
};
use rand::Rng;
use serde::Deserialize;
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

group!({
    name: "general",
    options: {},
    commands: [ping, announce]
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

#[derive(Deserialize, Debug)]
struct Config {
    //TODO: Validate function
    token: String,
}

fn load_config(p: &Path) -> Option<Config> {
    //TODO: Result
    if !p.exists() {
        return None;
    }

    let data = std::fs::read(p).ok()?;
    let config: Config = toml::from_slice(&data).ok()?;
    Some(config)
}

fn main() {
    println!("[INFO] Loading Config.toml...");
    let config = load_config(Path::new("./Config.toml")).expect("Could not load Config.toml");

    let mut client = Client::new(&config.token, Handler).expect("Error creating client");

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~"))
        .group(&GENERAL_GROUP)
        .help(&HELP)
        .on_dispatch_error(|_, msg, error| {
            println!("[ERROR] {:?}{}", error, msg.content);
        });
    //.help(help_commands::plain)

    client.with_framework(framework);

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
    let _my_stop = stop.clone();
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
    handle.join().unwrap();
}
