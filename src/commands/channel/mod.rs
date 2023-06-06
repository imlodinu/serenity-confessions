use tracing::{info, warn};

// this is a blank struct initialised in main.rs and then imported here
use crate::{operations, Data};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

use super::super::operations::channels::ChannelUse;

#[poise::command(slash_command, prefix_command, guild_only = true)]
pub async fn set_channel(
    ctx: Context<'_>,
    #[description = "Usage of the channel"] channel_use: ChannelUse,
) -> Result<(), Error> {
    let channel_result = operations::channels::add_channel_for_guild(
        &ctx.data().database,
        ctx.guild_id().unwrap().0,
        ctx.channel_id().0,
        channel_use.clone(),
    )
    .await;
    let response = match channel_result {
        Ok(_) => format!("Set channel usage to {}!", channel_use),
        Err(e) => e.to_string(),
    };
    if let Err(why_discord_say) = ctx.say(response).await {
        warn!("Error sending message: {:?}", why_discord_say);
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only = true)]
pub async fn get_channels(ctx: Context<'_>) -> Result<(), Error> {
    let channels_result = operations::channels::get_channels_in_guild(
        &ctx.data().database,
        ctx.guild_id().unwrap().0,
    )
    .await;
    let response = match channels_result {
        Ok(r) => serde_json::to_string(&r).unwrap_or("Error serialising channels".to_owned()),
        Err(e) => e.to_string(),
    };
    if let Err(why) = ctx.say(response).await {
        info!("Error sending message: {:?}", why);
    }
    Ok(())
}
