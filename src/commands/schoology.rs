use std::sync::Arc;
use serenity::framework::standard::{Args, CommandOptions, Command};
use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::framework::standard::CommandGroup;

use chrono::{TimeZone};

use super::super::schoology::{Api, GroupList, UserList, UpdateList};


pub fn get_group() -> CommandGroup{
	let mut group = CommandGroup::default();
	return group;
}

pub struct Schoology {
	options: Arc<CommandOptions>,
	api: Api,
}

impl Schoology {
	pub fn new(token: String, secret: String) -> Self {
		let mut opts = CommandOptions::default();
		opts.allowed_roles.push(String::from("bot"));
		
		let cmd = Schoology {
			options: Arc::from(opts),
			api: Api::new(token, secret),
		};
		
		return cmd;
	}
}

impl Command for Schoology {
	fn execute(&self, _: &mut Context, msg: &Message, mut args: Args) -> Result<(), serenity::framework::standard::CommandError> {
		let sub = args.single::<String>().unwrap();
		let arg = args.single::<String>().unwrap();
		
		match sub.as_ref() {
			"groups" => {
				match arg.as_ref() {
					"list" => {
						let mut start = 0;
						let mut limit = 3;
						
						if let (Ok(user_start), Ok(user_limit)) = (args.single::<usize>(), args.single::<usize>()){
							start = user_start;
							limit = user_limit;
						}
						
						let groups = self.api.get_groups(start, limit).unwrap();
						msg.channel_id.say(format_groups(&groups))?;
					},
					"info" => {
						if let Ok(id) = args.single::<String>(){
							let group = self.api.get_group(&id);
							msg.channel_id.say(&format!("{:#?}", group.unwrap()))?;
						}
					},
					"updates" => {
						if let Ok(id) = args.single::<String>(){
							let updates = self.api.get_group_updates(&id, 0, 3).unwrap();
							msg.channel_id.say(format_updates(&updates))?;
						}
					}
					_=> {
					
					}
				}
			},
			"users" => {
				match arg.as_ref() {
					"list" => {
						let mut start = 0;
						let mut limit = 3;
						
						if let (Ok(user_start), Ok(user_limit)) = (args.single::<usize>(), args.single::<usize>()){
							start = user_start;
							limit = user_limit;
						}
						
						let users = self.api.get_users(start, limit).unwrap();
						msg.channel_id.say(&format_users(&users))?;
					},
					_=> {
					
					}
				}
			},
			_=> {
			
			}
		}
		return Ok(());
	}
	
	fn options(&self) -> Arc<CommandOptions> {
		return self.options.clone();
	}
}

fn format_groups(groups: &GroupList) -> String{
	let mut ret = String::from("__**Groups:**__\n\n");
		
	for group in &groups.group{
		ret += &format!("__***{}***__\n", &group.title);
		
		ret += &format!("\t__ID:__ {}\n", &group.id);
		if group.description.len() > 0 {
			ret += &format!("\t__Description:__ {}\n", &group.description);
		}
		
		ret += "\n";
	}
	
	return ret;
}

fn format_users(users: &UserList) -> String {
	let mut ret = String::from("__**Users**__\n\n");
		
	for user in &users.user{
		ret += &format!("__***{} {} {} {}***__\n", &user.name_title, &user.name_first, &user.name_middle.as_ref().unwrap_or(&String::new()), &user.name_last);
		
		ret += &format!("\t__ID:__ {}\n", &user.id);
		if let Some(ref gender) = user.gender {
			ret += &format!("\t__Gender:__ {}\n", &gender);
		}
		
		if let Some(ref position) = user.position {
			ret += &format!("\t__Position:__ {}\n", &position);
		}
		
		ret += "\n";
	}
	
	return ret;
}

fn format_updates(updates: &UpdateList) -> String{
	let mut ret = String::from("__**Updates**__\n\n");
		
	for update in &updates.update{
		ret += &format!("__***Update***__\n");
		ret += &format!("\t__ID:__ {}\n", &update.id);
		ret += &format!("\t__Author:__ {}\n", &update.uid);
		ret += &format!("\t__Date:__ {}\n", chrono::Local.timestamp(update.created as i64, 0).to_rfc2822());
		ret += &format!("\t__Likes:__ {}\n", &update.likes);
		ret += &format!("\t__Comments:__ {}\n", &update.num_comments);
		ret += &format!("\t__Body:__ {}\n", &update.body);
		ret += "\n";
	}
	
	return ret;
}
