use tracing::info;

// this is a blank struct initialised in main.rs and then imported here
use crate::{auth, operations, Data};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub mod channel;
pub mod confessions;

#[poise::command(slash_command, prefix_command, guild_only = true)]
pub async fn print(ctx: Context<'_>) -> Result<(), Error> {
    let models = operations::guild::get_guilds(&ctx.data().database)
        .await
        .unwrap();
    if let Err(why) = ctx.say(serde_json::to_string(&models)?).await {
        info!("Error sending message: {:?}", why);
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only = true)]
pub async fn initialise(ctx: Context<'_>) -> Result<(), Error> {
    let auth_res =
        auth::respond_based_on_auth_context(&ctx, auth::Auth::Role(1114178684648165387.into()))
            .await;
    match auth_res {
        Ok(authorised) => {
            if !authorised {
                return Ok(());
            }
        }
        Err(_) => return Ok(()),
    };
    let db = ctx.data().database.clone();
    let this_guild = ctx.guild_id().unwrap().0;
    let found_guild = operations::guild::get_guild(&db, this_guild).await;
    let response = match found_guild {
        Ok(guild_model) => {
            format!("Guild({:#x}) already initialised.", guild_model.id)
        }
        Err(_) => {
            let add_result = operations::guild::add_or_nothing_guild(&db, this_guild).await;
            match add_result {
                Ok(_) => format!("Added guild({:#x}).", this_guild),
                Err(e) => e.to_string(),
            }
        }
    };
    if let Err(why_discord_say) = ctx.say(response).await {
        info!("Error sending message: {:?}", why_discord_say);
    }
    Ok(())
}
