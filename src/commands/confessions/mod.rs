use poise::{execute_modal, serenity_prelude as serenity, Modal};
use tracing::info;

use std::hash::Hasher;
use std::mem;
use twox_hash::XxHash64;

// this is a blank struct initialised in main.rs and then imported here
use crate::{
    auth,
    operations::{self, confession_guild_hashes},
    Data,
};

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
    content: String,
}

pub struct ConfessionInfo {
    author: serenity::User,
    content: String,
    image: Option<serenity::Attachment>,
}

fn to_user(col: u64) -> u32 {
    const MAX: u64 = 16_777_215; // Maximum color value (0xFFFFFF)
    return unsafe { mem::transmute::<u64, [u32; 2]>(col % MAX) }[0];
}

pub async fn get_hash_from_user(db: &Context<'_>, user: serenity::UserId) -> u32 {
    let guild_confession_hash = confession_guild_hashes::get_or_new_guild_hash(
        &db.data().database,
        db.guild_id().unwrap().0,
    )
    .await;
    let mut hasher = XxHash64::with_seed(guild_confession_hash.unwrap().hash);
    hasher.write_u64(user.0);
    to_user(hasher.finish())
}

pub async fn post_confession(
    ctx: &Context<'_>,
    target_channel: serenity::ChannelId,
    info: ConfessionInfo,
) {
    let show_id = get_hash_from_user(ctx, info.author.id).await;
    if let Err(why) = target_channel
        .send_message(&ctx, move |m| {
            m.embed(|embed| {
                embed
                    .description(info.content)
                    .author(|a| a.name(format!("[{:x}]", show_id)))
                    .colour(show_id);
                if let Some(image) = info.image {
                    embed.image(image.url.clone());
                }
                embed
            })
        })
        .await
    {
        println!("Error sending message: {:?}", why);
    }
}

pub async fn send_verify_confession(
    ctx: &Context<'_>,
    target_channel: serenity::ChannelId,
    info: ConfessionInfo,
) {
    let vetting_channel = operations::channels::get_channels_in_guild_with_use(
        &ctx.data().database,
        ctx.guild_id().unwrap().0,
        ChannelUse::Vetting,
    )
    .await;
    if let Err(why) = vetting_channel {
        if let Err(why_msg) = ctx
            .send(|builder| {
                builder
                    .content(format!(
                        "Error getting vetting channel: {:?}",
                        why.to_string()
                    ))
                    .ephemeral(true)
                    .reply(true)
            })
            .await
        {
            println!("Error sending message: {:?}", why_msg);
        }
        return;
    }
    let vetting_channels = vetting_channel.unwrap();
    match vetting_channels.get(0) {
        Some(channel_model) => {
            let channel_id = channel_model.id;
            let channel_find_result = ctx
                .guild()
                .unwrap()
                .channels
                .into_iter()
                .find(|(id, _)| id.0 == channel_id);
            if let None = channel_find_result {
                if let Err(why_msg) = ctx
                    .send(|builder| {
                        builder
                            .content(format!("Error getting vetting channel"))
                            .ephemeral(true)
                            .reply(true)
                    })
                    .await
                {
                    println!("Error sending message: {:?}", why_msg);
                }
                return;
            }
            let show_id = get_hash_from_user(ctx, info.author.id).await;
            if let Err(why) = channel_find_result
                .unwrap()
                .0
                .send_message(&ctx, |message| {
                    message
                        .content(format!("Confession going to <@&{}>", target_channel.0))
                        .embed(|embed| {
                            embed
                                .description(info.content)
                                .author(|a| a.name(format!("[{:x}]", show_id)))
                                .colour(show_id);
                            if let Some(image) = info.image {
                                embed.image(image.url.clone());
                            }
                            embed
                        })
                })
                .await
            {
                println!("Error sending message: {:?}", why);
            }
        }
        None => {
            if let Err(why_msg) = ctx
                .send(|builder| {
                    builder
                        .content(format!(
                            "There is no vetting channel set. Use `/set_vetting` to set one."
                        ))
                        .ephemeral(true)
                        .reply(true)
                })
                .await
            {
                println!("Error sending message: {:?}", why_msg);
            }
        }
    }
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
    let mut content = input_content.or(match &input_image {
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
            }
            poise::Context::Prefix(_) => None,
        };
    };
    // get a modal to send to the user
    let response = match channel_usage_result {
        Ok(channel_type) => {
            match channel_type == ChannelUse::Confession {
                true => {
                    // post_confession(&ctx, channel, ConfessionInfo { 
                    //     author: ctx.author().clone(), 
                    //     content: content.unwrap_or("?".to_owned()), 
                    //     image: input_image }).await;
                    // ConfessionInfo { 
                    //     author: ctx.author().clone(), 
                    //     content: content.unwrap_or("?".to_owned()), 
                    //     image: input_image };
                    send_verify_confession(
                        &ctx,
                        channel,
                        ConfessionInfo {
                            author: ctx.author().clone(),
                            content: content.unwrap_or("?".to_owned()), 
                            image: input_image 
                        }).await;
                    format!("Your confession has been sent to be vetted.")
                },
                false => format!("This channel (<#{}>) is not for confessing. Use `/list` to find places to confess.", ctx.channel_id()),
            }
        }
        Err(e) => format!(
            "Error getting channel usage: {}\nYour confession has not been processed.",
            e.to_string()
        ),
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

#[poise::command(slash_command, prefix_command, guild_only = true)]
pub async fn set_vetting(ctx: Context<'_>) -> Result<(), Error> {
    let auth_res = auth::respond_based_on_auth_context(&ctx, auth::Auth::Admin).await;
    if let Err(_) = auth_res {
        return Ok(());
    } else if let Ok(authorised) = auth_res {
        if !authorised {
            return Ok(());
        }
    };
    // check if there's another confession vetting channel
    let channels_result = operations::channels::get_channels_in_guild_with_use(
        &ctx.data().database,
        ctx.guild_id().unwrap().0,
        ChannelUse::Vetting,
    )
    .await;
    match channels_result {
        Ok(channels) => {
            if channels.len() > 0 {
                ctx.say(format!("There is already a vetting channel: <#{}>\nSet that one to none use before trying again.", channels[0].id)).await?;
                return Ok(());
            }
            if let Err(why) = super::channel::set_channel(&ctx, ChannelUse::Vetting).await {
                ctx.say(format!("Error setting channel: {}", why.to_string()))
                    .await?;
            }
            Ok(())
        }
        Err(e) => {
            ctx.say(format!("Error getting channel: {}", e.to_string()))
                .await?;
            return Ok(());
        }
    }
}

#[poise::command(slash_command, prefix_command, guild_only = true)]
pub async fn set_confessing(ctx: Context<'_>) -> Result<(), Error> {
    let auth_res = auth::respond_based_on_auth_context(&ctx, auth::Auth::Admin).await;
    if let Err(_) = auth_res {
        return Ok(());
    } else if let Ok(authorised) = auth_res {
        if !authorised {
            return Ok(());
        }
    };
    super::channel::set_channel(&ctx, ChannelUse::Confession).await
}
