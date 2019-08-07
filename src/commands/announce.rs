use serenity::{
    client::Context,
    framework::standard::{
        Args,
        Command,
        CommandOptions,
    },
    model::channel::{
        ChannelType,
        Message,
    },
    CACHE,
};
use std::sync::Arc;

pub struct Announce {
    options: Arc<CommandOptions>,
}

impl Announce {
    pub fn new() -> Self {
        let mut opts = CommandOptions::default();
        opts.allowed_roles.push("bot".to_string()); //TODO: Admin only
        opts.desc = Some(String::from("Broadcast a message to robotics members"));
        opts.usage = Some(String::from(r#""<announcement>""#));
        opts.example = Some(String::from(
            r#"~announce "Hello! This is an announcement!""#,
        ));
        opts.min_args = Some(1);
        opts.max_args = Some(1);

        let cmd = Announce {
            options: Arc::from(opts),
        };

        return cmd;
    }
}

impl Command for Announce {
    fn execute(
        &self,
        _: &mut Context,
        msg: &Message,
        mut args: Args,
    ) -> Result<(), serenity::framework::standard::CommandError> {
        let announcement = args.single_quoted::<String>().unwrap();
        announce_discord(&announcement);
        return Ok(());
    }

    fn options(&self) -> Arc<CommandOptions> {
        return self.options.clone();
    }
}

pub fn announce_discord(announcement: &str) {
    let guilds = &CACHE.read().guilds;
    if guilds.len() > 0 {
        for value in guilds.values() {
            let guild = value.read();
            for value in guild.channels.values() {
                let channel = value.read();
                if channel.name == "announcements" && channel.kind == ChannelType::Text {
                    println!(
                        r#"[INFO] Announcing "{}" to discord channel "{}/{}""#,
                        announcement, guild.name, channel.name
                    );
                    channel.say(announcement);
                }
            }
        }
    }
}
