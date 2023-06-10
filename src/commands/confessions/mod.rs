use ::serenity::futures::StreamExt;
use anyhow::anyhow;
use poise::{execute_modal, serenity_prelude as serenity, Modal};
use serde::{Deserialize, Serialize};
use tracing::info;

use std::mem;
use std::{hash::Hasher, time::Duration};
use twox_hash::XxHash64;

// this is a blank struct initialised in main.rs and then imported here
use crate::{
    auth, button,
    operations::{self, confession_guild_hashes},
    Data,
};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
type FrameworkContext<'a> = poise::FrameworkContext<'a, Data, Error>;

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfessionInfo {
    author: serenity::User,
    content: String,
    image: Option<String>,
}

fn to_user(col: u64) -> u32 {
    const MAX: u64 = 16_777_215; // Maximum color value (0xFFFFFF)
    return unsafe { mem::transmute::<u64, [u32; 2]>(col % MAX) }[0];
}

pub async fn get_guild_confession_hash(db: &sea_orm::DatabaseConnection, guild_id: u64) -> u64 {
    let guild_confession_hash = confession_guild_hashes::get_or_new_guild_hash(&db, guild_id).await;
    guild_confession_hash.unwrap().hash
}

pub fn get_hash_from_user(guild_confession_hash: u64, user: serenity::UserId) -> u32 {
    let mut hasher = XxHash64::with_seed(guild_confession_hash);
    hasher.write_u64(user.0);
    to_user(hasher.finish())
}

pub async fn send_verify_confession(
    ctx: Context<'_>,
    target_channel: serenity::ChannelId,
    info: ConfessionInfo,
) {
    let guild = ctx.guild_id().unwrap();
    let vetting_channel = operations::channels::get_channels_in_guild_with_use(
        &ctx.data().database,
        guild.0,
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
            let channel_id = serenity::ChannelId::from(channel_model.id);
            let show_id = get_hash_from_user(
                get_guild_confession_hash(&ctx.data().database, guild.0).await,
                info.author.id,
            );
            if let Err(why) = channel_id
                .send_message(&ctx, |message| {
                    message
                        .content(format!("Confession going to <#{}>", target_channel.0))
                        .embed(|embed| {
                            embed
                                .description(&info.content)
                                .author(|a| a.name(format!("[{:x}]", show_id)))
                                .colour(show_id);
                            if let Some(image) = &info.image {
                                embed.image(image);
                            }
                            embed
                        })
                        .components(|components| {
                            components.create_action_row(|action_row| {
                                action_row
                                    .add_button(
                                        serenity::CreateButton::default()
                                            .label("Approve")
                                            .style(serenity::ButtonStyle::Success)
                                            .custom_id(
                                                button::ConfessionButton::ApproveConfession((
                                                    info.author.id,
                                                    target_channel,
                                                ))
                                                .to_string(),
                                            )
                                            .to_owned(),
                                    )
                                    .add_button(
                                        serenity::CreateButton::default()
                                            .label("Deny")
                                            .style(serenity::ButtonStyle::Danger)
                                            .custom_id(
                                                button::ConfessionButton::DenyConfession
                                                    .to_string(),
                                            )
                                            .to_owned(),
                                    )
                            })
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
                    send_verify_confession(
                        *ctx,
                        channel,
                        ConfessionInfo {
                            author: ctx.author().clone(),
                            content: content.unwrap_or("?".to_owned()), 
                            image: input_image.map(|image| image.url)
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

pub const MOD_MAX_VOTES: usize = 5;

#[poise::command(prefix_command, guild_only = true)]
pub async fn vote_reveal(
    ctx: Context<'_>,
    #[description = "Reveal"] id: String,
) -> Result<(), Error> {
    let auth_res = auth::respond_based_on_auth_context(&ctx, auth::Auth::Admin).await;
    if let Err(_) = auth_res {
        return Ok(());
    } else if let Ok(authorised) = auth_res {
        if !authorised {
            return Ok(());
        }
    };

    let channel_id = ctx.channel_id();
    let the_mods = crate::commands::guild::get_guild_moderators(ctx).await;
    if let Err(why_no_mods) = the_mods {
        ctx.say(format!(
            "Error getting moderators: {}",
            why_no_mods.to_string()
        ))
        .await?;
        return Ok(());
    }
    let the_mods = the_mods.unwrap();

    let numbered_id = u32::from_str_radix(&id, 16);
    if let Err(_) = numbered_id {
        ctx.say(format!("Invalid ID: {}", id)).await?;
        return Ok(());
    }
    let guild_confession_hash =
        get_guild_confession_hash(&ctx.data().database, ctx.guild_id().unwrap().0).await;
    let found_out = match ctx
        .partial_guild()
        .await
        .unwrap()
        .members(ctx, None, None)
        .await
    {
        Ok(members) => {
            let found = members
                .into_iter()
                .find(|member| {
                    get_hash_from_user(guild_confession_hash, member.user.id)
                        == numbered_id.to_owned().unwrap()
                })
                .map(|member| member.user.id);
            match found {
                Some(id) => Ok(id),
                None => Err(anyhow!("Could not find user with ID: {}", id)),
            }
        }
        Err(e) => Err(anyhow!("Error getting members: {}", e.to_string())),
    };

    if let Err(why) = found_out {
        ctx.say(format!("Error: {}", why.to_string())).await?;
        return Ok(());
    }
    let found_user = found_out.unwrap();

    let reply_handle_res = ctx
        .send(|message| {
            message
                .reply(true)
                .content(format!("Reveal the user behind `{}`?", id))
                .components(|components| {
                    components.create_action_row(|row| {
                        row.create_button(|button| {
                            button
                                .custom_id(
                                    button::ConfessionRevealButton::RevealConfession(id.clone())
                                        .to_string(),
                                )
                                .label("Yes")
                        })
                        .create_button(|button| {
                            button
                                .custom_id(
                                    button::ConfessionRevealButton::KeepConfession.to_string(),
                                )
                                .label("No")
                        })
                    })
                })
        })
        .await;
    if let Err(e) = reply_handle_res {
        ctx.say(format!("Error sending message: {}", e.to_string()))
            .await?;
        return Ok(());
    }
    let the_mods_filter = the_mods.clone();

    let reply_handle_res = reply_handle_res.unwrap();
    let message = reply_handle_res.message().await.unwrap();
    let interaction_collector_builder = message.await_component_interactions(&ctx);
    let mut interaction_collector = interaction_collector_builder
        .channel_id(ctx.channel_id())
        .timeout(Duration::from_secs(30))
        .message_id(message.id)
        .filter(move |m| the_mods_filter.contains(&m.user.id))
        .build();
    let mut voted_for = vec![];
    let mut voted_against = vec![];
    let mut all_voted = vec![];
    while let Some(vote) = interaction_collector.next().await {
        let is_for = match button::ConfessionRevealButton::from_string(&vote.data.custom_id) {
            Some(vote_type) => match vote_type {
                button::ConfessionRevealButton::RevealConfession(_) => true,
                button::ConfessionRevealButton::KeepConfession => false,
                button::ConfessionRevealButton::None => false,
            },
            None => false,
        };
        let already_voted = all_voted.contains(&vote.user.id);
        if !already_voted {
            all_voted.push(vote.user.id);
        }
        if let Err(why) = vote
            .create_interaction_response(ctx, |message| {
                message.interaction_response_data(|response_data| {
                    response_data
                        .content(if already_voted {
                            format!("You already voted! (there are {} votes)", all_voted.len())
                        } else {
                            format!(
                                "You voted {} (there are {} votes)",
                                if is_for { "for" } else { "against" },
                                all_voted.len()
                            )
                        })
                        .ephemeral(true)
                })
            })
            .await
        {
            ctx.say(format!("Error sending message: {}", why.to_string()))
                .await?;
            return Ok(());
        }
        if !already_voted {
            (if is_for {
                &mut voted_for
            } else {
                &mut voted_against
            })
            .push(vote.user.id);
        }
    }
    if let Err(e) = message.into_owned()
        .edit(ctx, |message| {
            message.components(|components| components.set_action_rows(vec![]))
        })
        .await
    {
        ctx.say(format!("Error sending message: {}", e.to_string()))
            .await?;
        return Ok(());
    }

    let voted_for_string = voted_for
        .clone()
        .iter_mut()
        .map(|mci| format!("- <@{}>", mci))
        .collect::<Vec<String>>()
        .join("\n");
    let voted_against_string = voted_against
        .clone()
        .iter_mut()
        .map(|mci| format!("- <@{}>", mci))
        .collect::<Vec<String>>()
        .join("\n");
    if let Err(why) = ctx
        .say(format!(
            "These moderators voted for:\n{}\nThese ones voted against:\n{}",
            voted_for_string, voted_against_string
        ))
        .await
    {
        ctx.say(format!("Error sending message: {}", why.to_string()))
            .await?;
        return Ok(());
    };

    let mod_needed = ((the_mods.len() as f64) / 2f64).clamp(1.0, MOD_MAX_VOTES as f64);
    let proceed = voted_for.len() as f64 > mod_needed
        && voted_for.len() > 0
        && voted_for.len() > voted_against.len();
    if let Err(e) = channel_id
        .send_message(ctx, |message| {
            message.content(format!(
                "{}/{} moderators voted for. Needed {}. This is {}",
                voted_for.len(),
                the_mods.len(),
                mod_needed.trunc() as u32,
                if proceed { "approved" } else { "denied" }
            ))
        })
        .await
    {
        ctx.say(format!("Error sending message: {}", e.to_string()))
            .await?;
        return Ok(());
    }

    if proceed {
        if let Err(why) = ctx
            .send(|message| {
                message
                    .content(format!("Confessor is <@{}>.", found_user.0))
                    .allowed_mentions(|mentions| mentions.empty_parse())
            })
            .await
        {
            ctx.say(format!("Error sending message: {}", why.to_string()))
                .await?;
            return Ok(());
        }
    }

    Ok(())
}

pub async fn handle<'a>(
    ctx: &serenity::Context,
    ev: &poise::Event<'a>,
    _: FrameworkContext<'a>,
    data: &Data,
) -> Result<(), Error> {
    if let poise::Event::InteractionCreate { interaction } = ev {
        match interaction {
            serenity::Interaction::MessageComponent(component) => {
                match crate::button::ConfessionButton::from_string(&component.data.custom_id) {
                    Some(button_interaction) => {
                        let should_clear = match button_interaction {
                            crate::button::ConfessionButton::ApproveConfession(send_info) => {
                                let maybe_user = send_info.0.to_user(ctx).await;
                                if let Err(why_no_user) = maybe_user {
                                    println!("Error getting user: {:?}", why_no_user);
                                    return Ok(());
                                }
                                let user = maybe_user.unwrap();
                                let info_opt =
                                    component.message.embeds.get(0).map(|embed| ConfessionInfo {
                                        author: user,
                                        content: embed.description.clone().unwrap_or("".to_owned()),
                                        image: embed
                                            .image
                                            .clone()
                                            .map(|embed_image| embed_image.url),
                                    });
                                let mut valid = false;
                                match info_opt {
                                    Some(info) => {
                                        let show_id = get_hash_from_user(
                                            get_guild_confession_hash(
                                                &data.database,
                                                component.guild_id.unwrap().0,
                                            )
                                            .await,
                                            info.author.id,
                                        );
                                        if let Err(why) = send_info
                                            .1
                                            .send_message(&ctx, move |m| {
                                                m.embed(|embed| {
                                                    embed
                                                        .description(info.content)
                                                        .author(|a| {
                                                            a.name(format!("[{:x}]", show_id))
                                                        })
                                                        .colour(show_id);
                                                    if let Some(image) = info.image {
                                                        embed.image(image);
                                                    }
                                                    embed
                                                })
                                            })
                                            .await
                                        {
                                            println!("Error sending message: {:?}", why);
                                        }
                                        if let Err(why) = component
                                            .create_interaction_response(&ctx.http, |response| {
                                                response.interaction_response_data(
                                                    |response_data| {
                                                        response_data.content(format!(
                                                            "Confession accepted by <@{}>",
                                                            component.user.id
                                                        ))
                                                    },
                                                )
                                            })
                                            .await
                                        {
                                            println!("Error sending message: {:?}", why);
                                        } else {
                                            valid = true;
                                        }
                                    }
                                    None => {
                                        valid = false;
                                    }
                                }
                                valid
                            }
                            crate::button::ConfessionButton::DenyConfession => {
                                let mut valid = false;
                                if let Err(why) = component
                                    .create_interaction_response(&ctx.http, |response| {
                                        response.interaction_response_data(|response_data| {
                                            response_data.content(format!(
                                                "Confession denied by <@{}>",
                                                component.user.id
                                            ))
                                        })
                                    })
                                    .await
                                {
                                    println!("Error sending message: {:?}", why);
                                } else {
                                    valid = true;
                                }
                                valid
                            }
                            _ => false,
                        };
                        if should_clear {
                            let mut message = component.message.clone();
                            if let Err(e) = message
                                .edit(&ctx.http, |message| {
                                    message.set_components(serenity::CreateComponents::default())
                                })
                                .await
                            {
                                println!("Error sending message: {:?}", e);
                            };
                        }
                    }
                    None => {}
                }
            }
            _ => {}
        }
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only = true, guild_cooldown = 5)]
pub async fn shuffle(ctx: Context<'_>) -> Result<(), Error> {
    match operations::confession_guild_hashes::shuffle_guild_hash(
        &ctx.data().database,
        ctx.guild_id().unwrap().0,
    )
    .await
    {
        Ok(_) => {
            ctx.say(format!("Shuffled!")).await?;
        }
        Err(e) => {
            ctx.say(format!("Error shuffling: {}", e.to_string()))
                .await?;
        }
    };
    Ok(())
}
