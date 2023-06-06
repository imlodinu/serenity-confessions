use tracing::info;

// this is a blank struct initialised in main.rs and then imported here
use crate::{operations, Data};

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
    let db = ctx.data().database.clone();
    let this_guild = ctx.guild_id().unwrap().0;
    let add_result = operations::guild::add_or_nothing_guild(&db, this_guild).await;
    let response = match add_result {
        Ok(_) => "Added guild to database!".to_owned(),
        Err(e) => e.to_string(),
    };
    if let Err(why_discord_say) = ctx.say(response).await {
        info!("Error sending message: {:?}", why_discord_say);
    }
    Ok(())
}
