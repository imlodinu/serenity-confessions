use anyhow::{Result, anyhow};
use migration::{OnConflict};
use sea_orm::{DatabaseConnection, EntityTrait, Set, InsertResult};
use tracing::{info, warn};

use crate::entity::guild;

pub async fn get_guilds(db: &DatabaseConnection) -> Option<Vec<guild::Model>> {
    guild::Entity::find().all(db).await.ok()
}

pub async fn add_or_nothing_guild(db: &DatabaseConnection, guild_id: u64) -> Result<InsertResult<guild::ActiveModel>> {
    let this_guild = guild::ActiveModel {
        id: Set(guild_id),
    };
    let add_result = guild::Entity::insert(this_guild)
        .on_conflict(
            OnConflict::column(guild::Column::Id)
                .do_nothing()
                .to_owned(),
        )
        .exec(db)
        .await;
    if let Err(e) = &add_result {
        warn!("{:?}", e);
    }
    match add_result {
        Ok(r) => {
            info!("Added guild to database");
            Ok(r)
        }
        Err(e) => Err(anyhow!("Error adding guild to database: **{:?}**", e)),
    }
}