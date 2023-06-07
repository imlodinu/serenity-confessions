use anyhow::{anyhow, Result};
use migration::OnConflict;
use sea_orm::{DatabaseConnection, EntityTrait, InsertResult, QueryTrait, Set};
use tracing::{info, warn};

use crate::entity::guild;

pub async fn get_guilds(db: &DatabaseConnection) -> Option<Vec<guild::Model>> {
    guild::Entity::find().all(db).await.ok()
}

pub async fn get_guild(db: &DatabaseConnection, guild_id: u64) -> Result<guild::Model> {
    match guild::Entity::find_by_id(guild_id).one(db).await {
        Ok(g) => Ok(g.unwrap()),
        Err(e) => Err(anyhow!("Error getting guild from database: {:?}", e)),
    }
}

pub async fn add_or_nothing_guild(
    db: &DatabaseConnection,
    guild_id: u64,
) -> Result<InsertResult<guild::ActiveModel>> {
    let this_guild = guild::ActiveModel { id: Set(guild_id) };
    let add_result = guild::Entity::insert(this_guild.clone())
        .on_conflict(
            OnConflict::column(guild::Column::Id)
                .update_column(guild::Column::Id)
                .to_owned(),
        )
        .exec(db)
        .await;
    match add_result {
        Ok(r) => {
            info!("Added guild to database");
            Ok(r)
        }
        Err(e) => Err(anyhow!("Error adding guild to database: {:?}", e)),
    }
}
