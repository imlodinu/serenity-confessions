use tracing::info;
use poise::serenity_prelude as serenity;

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
    #[description = "Content"] content: Option<String>,
    #[description = "An image"] image: Option<serenity::Attachment>,
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
                true => format!("Your confession is on it's way to be vetted!"),
                false => format!("This channel (<@&{}>) is not for confessing. Use `/list` to find places to confess.", ctx.channel_id()),
            }
        },
        Err(e) => format!("Error getting channel usage: {}\nYour confession has not been processed.", e.to_string()),
    };
    if let Err(why) = ctx
        .send(|builder| builder.content(response).ephemeral(true).reply(true))
        .await
    {
        info!("Error sending message: {:?}", why);
    }
    Ok(())
}
