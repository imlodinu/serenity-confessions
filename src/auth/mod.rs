use std::borrow::Cow;

use anyhow::{anyhow, Result};
use poise::serenity_prelude as serenity;

use crate::Data;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[allow(dead_code)]
#[derive(Clone, Debug, Copy)]
pub enum Auth {
    User,
    Admin,
    Role(serenity::RoleId),
}

pub async fn is_user_authorised_for_action(
    ctx: &Context<'_>,
    member: Cow<'_, serenity::Member>,
    required: Auth,
) -> Result<bool> {
    match required {
        Auth::User => Ok(true),
        Auth::Admin => Ok(member.permissions(&ctx).unwrap().manage_guild()),
        Auth::Role(needed_role_id) => {
            match member.user.has_role(ctx, ctx.guild_id().unwrap(), needed_role_id).await {
                Ok(has_role) => Ok(has_role),
                Err(e) => Err(anyhow!(e.to_string())),
            }
        }
    }
}

pub async fn respond_based_on_auth_context(ctx: &Context<'_>, required: Auth) -> Result<bool> {
    match is_user_authorised_for_action(&ctx, ctx.author_member().await.unwrap(), required).await {
        Ok(allowed) => match allowed {
            true => Ok(true),
            false => {
                let formatted = match required {
                    Auth::User => "`user`".to_owned(),
                    Auth::Admin => "`admin`".to_owned(),
                    Auth::Role(id) => format!("<@&{}>", id.0),
                };
                match ctx
                    .send(|builder| {
                        builder.reply(true).content(format!(
                            "You are not authorised to use this command. Requires {}",
                            formatted
                        ))
                    })
                    .await
                {
                    Ok(_) => Ok(false),
                    Err(e) => Err(anyhow!(e.to_string())),
                }
            }
        },
        Err(e) => Err(anyhow!(e.to_string())),
    }
}
