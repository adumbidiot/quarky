extern crate serenity;
extern crate rand;
extern crate serde_json;
extern crate clokwerk;

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

fn get_token() -> Option<String>{
	if let Ok(mut file) = File::open("token.txt"){
		let mut buffer = String::new();
		if file.read_to_string(&mut buffer).is_ok(){
			return Some(buffer);
		}
	}
	
	if let Ok(token) = std::env::var("DISCORD_TOKEN"){
		return Some(token);
	}
	
	return None;
}



fn main(){
	println!("[INFO] Locating Token...");
	let token = get_token().expect("Could not find Token");
	
	let mut client = Client::new(&token, Handler).expect("Error creating client");
	
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
	
	scheduler
		.every(Monday)
		.at("12:00:00")
		.run(||{
			//TODO: Ensure client is started and connected before running. Maybe manually start/manage own thread?
			announce_discord("Robotics Club after school today!");
		});
	scheduler
		.every(Tuesday)
		.at("12:00:00")
		.run(||{
			announce_discord("Robotics Club during plus and lunch today!");
		});
	scheduler
		.every(Thursday)
		.at("12:00:00")
		.run(||{
			announce_discord("Robotics Club during plus and lunch today!");
		});	
	scheduler
		.every(Friday)
		.at("12:00:00")
		.run(||{
			announce_discord("Robotics Club after school today!");
		});
	
	let event_thread_handle = scheduler.watch_thread(Duration::from_secs(60 * 5));
	
	println!("[INFO] Logging in...");
	if let Err(why) = client.start() {
        println!("[ERROR] {:?}", why);
    }
	
	//println!("[INFO] Shutting down...");
}
