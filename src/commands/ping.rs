use std::sync::Arc;
use serenity::framework::standard::{Args, CommandOptions, Command};
use serenity::client::Context;
use serenity::model::channel::Message;

pub struct Ping {
	options: Arc<CommandOptions>,
}

impl Ping {
	pub fn new() -> Self {
		let mut opts = CommandOptions::default();
		opts.allowed_roles.push("bot".to_string());
		
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