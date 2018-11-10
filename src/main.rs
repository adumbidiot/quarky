extern crate serenity;
extern crate rand;
extern crate serde_json;
extern crate clokwerk;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate toml;


mod commands;

use serenity::client::{Client, EventHandler, Context};
use serenity::framework::standard::StandardFramework; 
use serenity::framework::standard::{help_commands};
use serenity::model::gateway::{Ready, Game};
use serenity::model::channel::Message;

use std::fs::File;
use std::io::Read;
use std::time::Duration;

use rand::Rng;

use clokwerk::Scheduler;
use clokwerk::Interval::{Monday, Tuesday, Thursday, Friday};

use std::path::Path;

use self::commands::announce::announce_discord;

struct Handler;

impl EventHandler for Handler {
	fn ready(&self, ctx: Context, ready: Ready) {
		let random_number: u8 = rand::thread_rng().gen_range(0, 2);
		match random_number {
			0 => {
				ctx.set_game(Game::playing("with my tail"));
			},
			_ => {
				ctx.set_game(Game::listening("hooman noises"));
			}
		}
		
		println!("[INFO] Choosing Game State {}", random_number);
        println!("[INFO] {} is connected!", ready.user.name);
		
		
		//Wait for cache to populate...
		std::thread::sleep(Duration::from_millis(1000));
		//Things that need the cache...
    }
	
	fn message(&self, ctx: Context, msg: Message){

	}
}

#[derive(Deserialize, Debug)]
struct Config { //TODO: Validate function
	token: String
}

fn load_config(p: &Path) -> Option<Config>{ //TODO: Result
	if !p.exists() {
		return None;
	}
	
	let data = std::fs::read(p).ok()?;
	let config: Config = toml::from_slice(&data).ok()?;
	return Some(config);
}


fn main(){
	println!("[INFO] Loading Config.toml...");
	let config = load_config(Path::new("./Config.toml")).expect("Could not load Config.toml");
	
	let mut client = Client::new(&config.token, Handler).expect("Error creating client");
	
	let framework = StandardFramework::new()
		.configure(|c|{
			c.prefix("~")
		})
		.cmd("ping", commands::ping::Ping::new())
		.cmd("announce", commands::announce::Announce::new())
		.help(help_commands::plain)
		.on_dispatch_error(|_, msg, error|{
			println!("[ERROR] {:?}{}", error, msg.content);
		});
		
	client.with_framework(framework);
	
	println!("[INFO] Starting Event Scheduler...");
	//TODO: Wrap in arc and rwlock for dynamically adding and removing events
	let mut scheduler = Scheduler::new();
	const AFTER_SCHOOL_ANNOUNCE: &str = "@everyone Robotics Club after school today!";
	const LUNCH_ANNOUNCE: &str = "@everyone Robotics Club during plus and lunch today!";
	const NOON: &str = "12:00:00";
	scheduler
		.every(Monday)
		.at(NOON)
		.run(||{
			//TODO: Ensure client is started and connected before running. Maybe manually start/manage own thread?
			announce_discord(AFTER_SCHOOL_ANNOUNCE);
		});
	scheduler
		.every(Tuesday)
		.at(NOON)
		.run(||{
			announce_discord(LUNCH_ANNOUNCE);
		});
	scheduler
		.every(Thursday)
		.at(NOON)
		.run(||{
			announce_discord(LUNCH_ANNOUNCE);
		});	
	scheduler
		.every(Friday)
		.at(NOON)
		.run(||{
			announce_discord(AFTER_SCHOOL_ANNOUNCE);
		});
	
	let event_thread_handle = scheduler.watch_thread(Duration::from_secs(60 * 5));
	
	println!("[INFO] Logging in...");
	if let Err(why) = client.start() {
        println!("[ERROR] {:?}", why);
    }
	
	println!("[INFO] Shutting down...");
}
