use ::serenity::http::CacheHttp;
use poise::serenity_prelude as serenity;
use tracing::info;

// this is a blank struct initialised in main.rs and then imported here
use crate::{auth, operations, Data};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub async fn get_guild_moderators(ctx: Context<'_>) -> anyhow::Result<Vec<serenity::UserId>> {
    let db = ctx.data().database.clone();
    let this_guild = ctx.guild_id().unwrap().0;
    let found_admin_role = operations::guild::get_guild_mod_role(&db, this_guild).await;
    match found_admin_role {
        Ok(admin_role) => match admin_role {
            Some(valid_admin_role) => {
                let role_ref = &serenity::RoleId(valid_admin_role);
                let guild = ctx.partial_guild().await.unwrap();
                let members = guild.members(&ctx.http(), None, None).await.unwrap();
                let mut moderators = Vec::new();
                for member in members {
                    if member.roles.contains(role_ref) {
                        moderators.push(member.user.id);
                    }
                }
                Ok(moderators)
            }
            None => {
                return Ok(vec![]);
            }
        },
        Err(_) => Err(anyhow::anyhow!(
            "Guild not found. Have you used initialise?"
        )),
    }
}

#[poise::command(slash_command, prefix_command, guild_only = true)]
pub async fn set_mod_role(
    ctx: Context<'_>,
    #[description = "Id for moderators"] role: Option<serenity::RoleId>,
) -> Result<(), Error> {
    let auth_res = auth::respond_based_on_auth_context(&ctx, auth::Auth::Admin).await;
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
        Ok(mut guild_model) => {
            guild_model.admin_role = role.map(|r| r.0);
            let set_result = operations::guild::set_guild(&db, guild_model).await;
            match set_result {
                Ok(_) => {
                    match role {
                        Some(role_id) => format!("Set mod role to <@&{}>.", role_id),
                        None => format!("Cleared mod role."),
                    }
                }
                Err(e) => e.to_string(),
            }
        }
        Err(_) => {
            format!("Guild not found. Have you used initialise?")
        }
    };
    if let Err(why_discord_say) = ctx.say(response).await {
        info!("Error sending message: {:?}", why_discord_say);
    };
    Ok(())
}
