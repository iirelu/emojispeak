extern crate serenity;
extern crate unic_emoji_char;
extern crate regex;
#[macro_use] extern crate lazy_static;

use serenity::prelude::*;
use serenity::model::channel::Message;
use serenity::model::event::MessageUpdateEvent;
use serenity::model::guild::Guild;
use unic_emoji_char::is_emoji;
use regex::Regex;

const DISCORD_TOKEN: &str = "NDUxNDU5NTcwOTkzNDYzMjk3.DfCGkw.7T-NxcfPrcudVoILLbMElMrsHwY";
const CHAR_WHITELIST: &[char] = &[' '];
lazy_static! {
    static ref DISCORD_EMOJI_REGEX: Regex =
        Regex::new(r"<:[a-zA-Z0-9_]+:[0-9]+>").unwrap();
    static ref DISCORD_MENTION_REGEX: Regex =
        Regex::new(r"<@\d+>").unwrap();
    static ref ACTION_REGEX: Regex =
        Regex::new(r"\*.*?\*|_.*?_").unwrap();
    static ref URL_REGEX: Regex =
        Regex::new(r"https?://\S+").unwrap();
}

fn main() -> Result<(), serenity::Error> {
    let mut client = Client::new(DISCORD_TOKEN, Handler)?;
    client.start()?;
    Ok(())
}

struct Handler;

impl EventHandler for Handler {
    fn guild_create(&self, _: Context, guild: Guild, _: bool) {
        println!("connected to guild! name: '{}', members: {}",
            guild.name,
            guild.member_count);
    }

    fn message(&self, _: Context, msg: Message) {
        if has_role(&msg, "emojispeaker") {
            println!("emojispeaker message from {}: {}",
                msg.author.name,
                msg.content);
            if !is_emojispeech(&msg.content) {
                let _ = msg.delete();
            }
        }
    }

    fn message_update(&self, _: Context, msg: MessageUpdateEvent) {
        // urgh, have to redo all the has_role logic because different interface
        let has_role = || {
            let msg = msg.clone();
            let cache = &serenity::CACHE;
            let guild = {
                // it seriously does not work unless i do this one by one. wtf
                let g = cache.read().guild_channel(msg.channel_id)?;
                let g = g.read();
                g.guild_id
            };
            for role_id in cache.read().member(guild, msg.author?.id)?.roles {
                if cache.read().role(guild, role_id)?.name == "emojispeaker" {
                    return Some(true);
                }
            }
            Some(false)
        };
        match (has_role(), &msg.content) {
            (Some(true), Some(content)) => {
                if !is_emojispeech(&content) {
                    if let Some(channel) = serenity::CACHE.read()
                        .guild_channel(msg.channel_id)
                    {
                        let _ = channel.read().delete_messages(&[msg.id]);
                    }
                }
            }
            (_, _) => (),
        }
    }
}

fn has_role(msg: &Message, role_name: &str) -> bool {
    let guild = match msg.guild() {
        Some(guild) => guild,
        None => return false,
    };
    if let Some(member) = msg.member() {
        for role_id in &member.roles {
            if let Some(role) = guild.read().roles.get(&role_id) {
                if role.name == role_name {
                    return true;
                }
            }
        }
    };
    false
}

fn is_emojispeech(string: &str) -> bool {
    let string = DISCORD_EMOJI_REGEX.replace_all(string, "");
    let string = DISCORD_MENTION_REGEX.replace_all(&string, "");
    let string = ACTION_REGEX.replace_all(&string, "");
    let string = URL_REGEX.replace_all(&string, "");
    for chr in string.chars() {
        if is_emoji(chr) || CHAR_WHITELIST.contains(&chr) {
            continue;
        } else {
            return false;
        }
    }
    true
}
