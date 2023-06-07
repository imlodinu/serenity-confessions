use tracing::info;

// this is a blank struct initialised in main.rs and then imported here
use crate::{operations, Data};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

use super::super::operations::channels::ChannelUse;

#[poise::command(
    slash_command,
    prefix_command,
    description_localized("en-GB", "Confesses"),
    guild_only = true
)]
pub async fn confess(
    ctx: Context<'_>,
    #[description = "Context"] channel_use: ChannelUse,
) -> Result<(), Error> {
    let channel_usage_result = operations::channels::get_channel_use(
        &ctx.data().database,
        ctx.guild_id().unwrap().0,
        ctx.channel_id().0,
    )
    .await;
    let response = match channel_usage_result {
        Ok(channel_type) => {
            match channel_type == ChannelUse::ConfessionOut {
                true => format!("Confessing in channel {}", channel_use),
                false => format!("Channel {} is not for confessing", channel_use),
            }
        },
        Err(e) => format!("Error getting channel usage: {}", e.to_string()),
    };
    if let Err(why) = ctx.say(response).await {
        info!("Error sending message: {:?}", why);
    }
    Ok(())
}
