use ::serenity::http::CacheHttp;
use poise::serenity_prelude as serenity;
use tracing::info;

// this is a blank struct initialised in main.rs and then imported here
use crate::{auth, operations, Data};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(slash_command, prefix_command, guild_only = true)]
pub async fn ping_vc(
    ctx: Context<'_>,
    channel: Option<serenity::ChannelId>,
) -> Result<(), Error> {
    for (channel_id, channel_value) in ctx.guild().unwrap().channels {
        if let serenity::Channel::Guild(channel_value) = channel_value {
        let members = channel_value.members(ctx).await?;
        let is_looking_channel = if channel.is_some() {
            channel_id == channel.unwrap().0
        } else { members.iter().any(|x| x.user.id == ctx.author().id ) };
        if !is_looking_channel {
            continue;
        }
            if channel_value.kind == serenity::ChannelType::Voice {
                let mut ping_string = String::from("Pinging:\n");
                for member in members {
                    ping_string.push_str(&format!("<@{}> ", member.user.id));
                }
                ctx.say(ping_string).await?;
                return Ok(());
            } else {
                ctx.say("Channel not found").await?;
            }
        }
    }
    ctx.say("Channel not found").await?;
    Ok(())
}