use poise::serenity_prelude as serenity;
use shuttle_secrets::SecretStore;

mod commands;
mod router;
use router::build_router;
mod auth;
mod database;
mod entity;
mod operations;
mod util;

pub struct Data {
    database: sea_orm::DatabaseConnection,
}
pub struct BotService {
    discord_bot: poise::FrameworkBuilder<
        Data,
        Box<(dyn std::error::Error + std::marker::Send + Sync + 'static)>,
    >,
    router: axum::Router,
}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for BotService {
    async fn bind(mut self, addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        let router = self.router;

        let serve_router = axum::Server::bind(&addr).serve(router.into_make_service());
        tokio::select!(
            _ = self.discord_bot.run() => {},
            _ = serve_router => {}
        );

        Ok(())
    }
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> Result<BotService, shuttle_runtime::Error> {
    database::initialise(&secret_store);

    let discord_api_key = secret_store.get("DISCORD_TOKEN").unwrap();

    let discord_bot = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::reset_commands(),
                commands::initialise(),
                //
                commands::channel::get_channels(),
                //
                commands::confessions::confess(),
                commands::confessions::confess_to(),
                commands::confessions::set_vetting(),
                commands::confessions::set_confessing(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(".".into()),
                edit_tracker: Some(poise::EditTracker::for_timespan(
                    std::time::Duration::from_secs(3600),
                )),
                case_insensitive_commands: true,
                ..Default::default()
            },
            ..Default::default()
        })
        .token(discord_api_key)
        .intents(
            serenity::GatewayIntents::MESSAGE_CONTENT
                | serenity::GatewayIntents::GUILD_MESSAGES
                | serenity::GatewayIntents::DIRECT_MESSAGES,
        )
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    database: database::connect().await.unwrap(),
                })
            })
        });

    let router = build_router();

    Ok(BotService {
        discord_bot,
        router,
    })
}
