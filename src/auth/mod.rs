use anyhow::{anyhow, Result};
use poise::serenity_prelude as serenity;

use crate::Data;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[allow(dead_code)]
#[derive(Clone, Debug, Copy)]
pub enum Auth {
    Everyone,
    Admin,
    User(serenity::UserId),
    Role(serenity::RoleId),
}

pub async fn send_unauthorised_message(ctx: &Context<'_>, required: Auth) -> Result<()> {
    let formatted = match required {
        Auth::Everyone => "`everyone`".to_owned(),
        Auth::Admin => "`admin`".to_owned(),
        Auth::User(id) => format!("user to be <@{}>", id.0),
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
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(e.to_string())),
    }
}

pub async fn respond_based_on_auth_context(ctx: &Context<'_>, required: Auth) -> Result<bool> {
    let result = match required {
        Auth::Everyone => Ok(true),
        Auth::Admin => match ctx.partial_guild().await {
            Some(_) => Ok(ctx
                .author_member()
                .await
                .map(|v| {
                    v.permissions
                        .map(|permissions| permissions.manage_channels())
                })
                .unwrap_or(Some(false))
                .unwrap_or(false)),
            None => Ok(false),
        },
        Auth::User(user_id) => {
            let author_id = ctx.author().id;
            Ok(author_id == user_id)
        }
        Auth::Role(role_id) => match ctx.partial_guild().await {
            Some(_) => Ok(ctx
                .author_member()
                .await
                .map(|v| v.roles.iter().any(|role| role.0 == role_id.0))
                .unwrap_or(false)),
            None => Ok(false),
        },
    };
    if let Err(e) = result {
        send_unauthorised_message(ctx, required).await?;
        return Err(e);
    } else if let Ok(v) = result {
        if v == false {
            send_unauthorised_message(ctx, required).await?;
        }
        return Ok(v);
    };
    return Ok(false);
}
