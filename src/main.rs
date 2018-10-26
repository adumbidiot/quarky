extern crate serenity;
extern crate rand;

use serenity::client::{Client, EventHandler, Context};
use serenity::framework::standard::{StandardFramework, Command, Args}; 
use serenity::framework::standard::CommandOptions;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::gateway::Game;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use rand::Rng;

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
    }
}

struct Ping {
	options: Arc<CommandOptions>,
}

impl Ping {
	pub fn new() -> Self {
		let mut opts = CommandOptions::default();
		opts.allowed_roles.push("Admin".to_string());
		
		let cmd = Ping {
			options: Arc::from(opts),
		};
		
		return cmd;
	}
}

impl Command for Ping {
	fn execute(&self, _: &mut Context, msg: &Message, _: Args) -> Result<(), serenity::framework::standard::CommandError> {
		msg.channel_id.say("pong")?;
		return Ok(());
	}
	
	fn options(&self) -> Arc<CommandOptions> {
		return self.options.clone();
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
	
	//&std::env::var("DISCORD_TOKEN").expect("token")
	return None;
}

fn main(){
	let token = get_token().expect("Could not find Token");
	
	let mut client = Client::new(&token, Handler).expect("Error creating client");
	
	let framework = StandardFramework::new()
		.configure(|c|{
			c.prefix("~")
		})
		.cmd("ping", Ping::new());
		
	client.with_framework(framework);
	
	println!("[INFO] Logging in...");
	
	if let Err(why) = client.start() {
        println!("[Error] {:?}", why);
    }
	
	//println!("[INFO] Shutting down...");
}
