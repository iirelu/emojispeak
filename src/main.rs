extern crate serenity;
extern crate unic_emoji_char;
extern crate regex;
#[macro_use] extern crate lazy_static;

use serenity::prelude::*;
use serenity::model::channel::Message;
use serenity::model::event::MessageUpdateEvent;
use serenity::model::guild::Guild;
use serenity::model::id::{ChannelId, UserId};
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
        if has_role(msg.channel_id, msg.author.id, "emojispeaker") {
            println!("emojispeaker message from {}: {}",
                msg.author.name,
                msg.content);
            if !is_emojispeech(&msg.content) {
                let _ = msg.delete();
            }
        }
    }

    fn message_update(&self, _: Context, msg: MessageUpdateEvent) {
        (|| {
            if has_role(msg.channel_id, msg.author?.id, "emojispeaker") {
                println!("emojispeaker edited message: {:?}", msg.content);
                if !is_emojispeech(&msg.content?) {
                    let _ = msg.channel_id.delete_message(msg.id);
                }
            }
            Some(())
        })();
    }
}

fn has_role(channel_id: ChannelId, user_id: UserId, role_name: &str) -> bool {
    let guild_id = match serenity::CACHE.read().guild_channel(channel_id) {
        Some(g) => g.read().guild_id,
        None => return false,
    };
    guild_id.member(user_id).ok()
        .and_then(|member| Some(member.roles()?.iter()
            .filter(|role| role.name == role_name)
            .count() > 0))
        .unwrap_or(false)
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
