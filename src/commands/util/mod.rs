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
    channel: serenity::ChannelId,
) -> Result<(), Error> {
    if let Ok(serenity::Channel::Guild(manifested_channel)) =  channel.to_channel(ctx).await {
        if manifested_channel.kind == serenity::ChannelType::Voice {
            let members = manifested_channel.members(ctx).await?;
            let mut ping_string = String::from("Pinging: ");
            for member in members {
                ping_string.push_str(&format!("<@{}> ", member.user.id));
            }
            ctx.say(ping_string).await?;
        } else {
            ctx.say("Channel not found").await?;
        }
    } else {
        ctx.say("Channel not found").await?;
    }
    Ok(())
}