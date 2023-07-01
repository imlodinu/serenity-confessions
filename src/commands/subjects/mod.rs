use ::serenity::http::CacheHttp;
use poise::serenity_prelude as serenity;
use tracing::info;

// this is a blank struct initialised in main.rs and then imported here
use crate::{auth, operations, Data};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// Set the valid subjects
#[poise::command(slash_command, prefix_command, guild_only = true)]
pub async fn add_subject(
    ctx: Context<'_>,
    #[description = "Subject"] subject: String,
) -> Result<(), Error> {
    let auth_res = auth::respond_based_on_auth_context(&ctx, auth::Auth::Admin).await;
    if let Err(_) = auth_res {
        return Ok(());
    } else if let Ok(authorised) = auth_res {
        if !authorised {
            return Ok(());
        }
    };

    let db = ctx.data().database.clone();
    let this_guild = ctx.guild_id().unwrap().0;
    if let Err(why) =
        operations::subjects::add_guild_subject(&db, this_guild, subject.clone()).await
    {
        ctx.say(format!("Error adding subject: {}", why)).await?;
    } else {
        ctx.say(format!("Added subject: {}", subject)).await?;
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only = true)]
pub async fn get_subjects(ctx: Context<'_>) -> Result<(), Error> {
    let db = ctx.data().database.clone();
    let this_guild = ctx.guild_id().unwrap().0;
    let subjects = operations::subjects::get_guild_subjects(&db, this_guild).await?;
    let subjects_string = subjects
        .into_iter()
        .map(|s| format!("- {}", s))
        .collect::<Vec<String>>()
        .join("\n");
    ctx.say(format!("Subjects:\n{}", subjects_string)).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only = true)]
pub async fn remove_subject(
    ctx: Context<'_>,
    #[description = "Subject"] subject: String,
) -> Result<(), Error> {
    let auth_res = auth::respond_based_on_auth_context(&ctx, auth::Auth::Admin).await;
    if let Err(_) = auth_res {
        return Ok(());
    } else if let Ok(authorised) = auth_res {
        if !authorised {
            return Ok(());
        }
    };

    let db = ctx.data().database.clone();
    let this_guild = ctx.guild_id().unwrap().0;
    if let Err(why) =
        operations::subjects::remove_guild_subject(&db, this_guild, subject.clone()).await
    {
        ctx.say(format!("Error removing subject: {}", why)).await?;
    } else {
        ctx.say(format!("Removed subject: {}", subject)).await?;
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only = true)]
pub async fn add_user_subjects(
    ctx: Context<'_>,
    #[description = "Subject delimited by space"] subjects: String,
    #[description = "User"] user: Option<serenity::Member>,
) -> Result<(), Error> {
    let db = ctx.data().database.clone();
    let this_guild = ctx.guild_id().unwrap().0;
    let author_user_id = ctx.author_member().await.unwrap().user.id.0;
    let user_id = user.map(|u| u.user.id.0).unwrap_or(author_user_id);

    if user_id != author_user_id {
        let auth_res = auth::respond_based_on_auth_context(&ctx, auth::Auth::Admin).await;
        if let Err(_) = auth_res {
            return Ok(());
        } else if let Ok(authorised) = auth_res {
            if !authorised {
                return Ok(());
            }
        };
    }

    let send_subjects = subjects
        .clone()
        .split(" ")
        .map(|s| s.to_owned())
        .collect::<Vec<String>>();
    if let Err(why) =
        operations::subjects::add_user_subjects(&db, this_guild, user_id, send_subjects.clone())
            .await
    {
        ctx.say(format!("Error adding subjects: {}", why)).await?;
    } else {
        let fmted = send_subjects
            .into_iter()
            .map(|x| format!("- {}", x))
            .collect::<Vec<String>>()
            .join("\n");
        ctx.say(format!("Added subjects:\n{}", fmted)).await?;
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only = true)]
pub async fn get_user_subjects(
    ctx: Context<'_>,
    #[description = "User"] user: Option<serenity::Member>,
) -> Result<(), Error> {
    let db = ctx.data().database.clone();
    let this_guild = ctx.guild_id().unwrap().0;
    let author_user_id = ctx.author_member().await.unwrap().user.id.0;
    let user_id = user.map(|u| u.user.id.0).unwrap_or(author_user_id);

    let subjects = operations::subjects::get_user_subjects(&db, this_guild, user_id).await?;
    let subjects_string = subjects
        .into_iter()
        .map(|s| format!("- {}", s))
        .collect::<Vec<String>>()
        .join("\n");
    ctx.say(format!("Subjects:\n{}", subjects_string)).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only = true)]
pub async fn remove_user_subjects(
    ctx: Context<'_>,
    #[description = "Subject delimited by space"] subjects: String,
    #[description = "User"] user: Option<serenity::Member>,
) -> Result<(), Error> {
    let db = ctx.data().database.clone();
    let this_guild = ctx.guild_id().unwrap().0;
    let author_user_id = ctx.author_member().await.unwrap().user.id.0;
    let user_id = user.map(|u| u.user.id.0).unwrap_or(author_user_id);

    if user_id != author_user_id {
        let auth_res = auth::respond_based_on_auth_context(&ctx, auth::Auth::Admin).await;
        if let Err(_) = auth_res {
            return Ok(());
        } else if let Ok(authorised) = auth_res {
            if !authorised {
                return Ok(());
            }
        };
    }

    let send_subjects = subjects
        .clone()
        .split(" ")
        .map(|s| s.to_owned())
        .collect::<Vec<String>>();
    if let Err(why) =
        operations::subjects::remove_user_subjects(&db, this_guild, user_id, send_subjects.clone())
            .await
    {
        ctx.say(format!("Error removing subjects: {}", why)).await?;
    } else {
        let fmted = send_subjects
            .into_iter()
            .map(|x| format!("- {}", x))
            .collect::<Vec<String>>()
            .join("\n");
        ctx.say(format!("Removed subjects:\n{}", fmted)).await?;
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only = true)]
pub async fn get_users_with_subject(
    ctx: Context<'_>,
    #[description = "Subject"] subject: String,
) -> Result<(), Error> {
    let db = ctx.data().database.clone();
    let this_guild = ctx.guild_id().unwrap().0;
    let users =
        operations::subjects::get_users_with_subject(&db, this_guild, subject.clone()).await?;
    let users_string = users
        .into_iter()
        .map(|s| format!("- <@{}>", s))
        .collect::<Vec<String>>()
        .join("\n");
    ctx.send(|builder| {
        builder
            .content(format!("Users:\n{}", users_string))
            .allowed_mentions(|allowed| allowed.empty_parse())
    })
    .await?;
    Ok(())
}
