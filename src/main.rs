extern crate serenity;
extern crate rand;
#[macro_use]
extern crate serde_json;
extern crate clokwerk;

use serenity::client::{Client, EventHandler, Context};
use serenity::framework::standard::StandardFramework; 
use serenity::model::gateway::{Ready, Game};

use std::fs::File;
use std::io::Read;
use std::time::Duration;

use rand::Rng;

use clokwerk::Scheduler;
use clokwerk::Interval::Monday;

mod commands;



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
		/*serenity::http::raw::send_message(223233237281931268, &json!({
			"content": "hi"
		}));*/
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
		.cmd("ping", commands::ping::Ping::new());
		
	client.with_framework(framework);
	
	println!("[INFO] Starting Event Scheduler...");
	//TODO: Wrap in arc and rwlock for dynamically adding and removing events
	let mut scheduler = Scheduler::new();
	scheduler
		.every(Monday)
		.at("12:00:00")
		.run(||{
			//TODO: Dynamically retrive channel "announcements" for EACH guild and play message
			//TODO: Ensure client is started and connected before running. Maybe manually start/manage own thread?
			serenity::http::raw::send_message(223233237281931268, &json!({
				"content": "Robotics Club after school today!"
			}));
		});
	let event_thread_handle = scheduler.watch_thread(Duration::from_millis(1000000));
	
	println!("[INFO] Logging in...");
	if let Err(why) = client.start() {
        println!("[Error] {:?}", why);
    }
	
	//println!("[INFO] Shutting down...");
}
