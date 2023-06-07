use poise::serenity_prelude as serenity;
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
    let confession_channels_result = operations::channels::get_channels_in_guild_with_use(
        &ctx.data().database,
        ctx.guild_id().unwrap().0,
        channel_use,
    )
    .await;
    let parsed_confession_channels_result =
        confession_channels_result.map(|c| c.into_iter().map(|c| c.id).collect::<Vec<u64>>());
    let response = match parsed_confession_channels_result {
        Ok(r) => serde_json::to_string(&r).unwrap_or("Error serialising channels".to_owned()),
        Err(e) => e.to_string(),
    };
    if let Err(why) = ctx.say(response).await {
        info!("Error sending message: {:?}", why);
    }
    Ok(())
}
