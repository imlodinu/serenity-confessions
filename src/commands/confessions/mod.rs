use tracing::info;
use poise::{serenity_prelude as serenity, Modal, execute_modal};

// this is a blank struct initialised in main.rs and then imported here
use crate::{operations, Data};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

use super::super::operations::channels::ChannelUse;

#[derive(Debug, Modal)]
#[name = "Input"]
struct ConfessionModal {
    #[name = "Confession content"] // Field name by default
    #[min_length = 5]
    #[max_length = 500]
    #[paragraph]
    content: String
}

pub async fn _confess_to(
    ctx: &Context<'_>,
    channel: serenity::ChannelId,
    input_content: Option<String>,
    input_image: Option<serenity::Attachment>,
) -> Result<(), Error> {
    let channel_usage_result = operations::channels::get_channel_use(
        &ctx.data().database,
        ctx.guild_id().unwrap().0,
        channel.0,
    )
    .await;
    let mut content = input_content.or(match input_image {
        Some(image) => Some(format!("[Filename: {}]", image.filename)),
        None => None,
    });
    if let None = content {
        content = match ctx {
            poise::Context::Application(app) => {
                let modal = execute_modal::<_, _, ConfessionModal>(*app, None, None).await;
                if let Ok(modal_result) = modal {
                    modal_result.map(|m| m.content)
                } else {
                    None
                }
            },
            poise::Context::Prefix(_) => None
        };
    };
    // get a modal to send to the user
    let response = match channel_usage_result {
        Ok(channel_type) => {
            match channel_type == ChannelUse::ConfessionOut {
                true => format!("Your confession is on it's way to be vetted!\n({})", content.unwrap_or("?".to_owned())),
                false => format!("This channel (<#{}>) is not for confessing. Use `/list` to find places to confess.", ctx.channel_id()),
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

#[poise::command(
    slash_command,
    prefix_command,
    description_localized("en-GB", "Confess to a channel"),
    description_localized("en-US", "Confess to a channel"),
    guild_only = true
)]
pub async fn confess_to(
    ctx: Context<'_>,
    #[description = "Channel to confess to"] channel: serenity::ChannelId,
    #[description = "Content"] content: Option<String>,
    #[description = "An image"] image: Option<serenity::Attachment>,
) -> Result<(), Error> {
    _confess_to(&ctx, channel, content, image).await
}

#[poise::command(
    slash_command,
    prefix_command,
    description_localized("en-GB", "Confesses to the current channel."),
    description_localized("en-US", "Confesses to the current channel."),
    guild_only = true
)]
pub async fn confess(
    ctx: Context<'_>,
    #[description = "Content"] content: Option<String>,
    #[description = "An image"] image: Option<serenity::Attachment>,
) -> Result<(), Error> {
    _confess_to(&ctx, ctx.channel_id(), content, image).await
}
