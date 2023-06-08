use anyhow::{anyhow, Result};
use rand::Rng;
use sea_orm::{DatabaseConnection, EntityTrait, Set};

use crate::entity::confession_guild_hashes;

pub async fn set_guild_hash(
    db: &DatabaseConnection,
    guild_id: u64,
    hash: u64,
) -> Result<confession_guild_hashes::Model> {
    let guild_hash = confession_guild_hashes::ActiveModel {
        guild_id: Set(guild_id),
        hash: Set(hash),
    };

    let guild_hash_result = confession_guild_hashes::Entity::insert(guild_hash)
        .exec(db)
        .await;

    match guild_hash_result {
        Ok(_) => Ok(confession_guild_hashes::Model { guild_id, hash }),
        Err(e) => Err(anyhow!("Error inserting guild hash into database: {:?}", e)),
    }
}

pub async fn get_or_new_guild_hash(
    db: &DatabaseConnection,
    guild_id: u64,
) -> Result<confession_guild_hashes::Model> {
    let guild_hash_result = confession_guild_hashes::Entity::find_by_id(guild_id)
        .one(db)
        .await;

    match guild_hash_result {
        Ok(guild_hash_opt) => {
            if let Some(guild_hash) = guild_hash_opt {
                Ok(guild_hash)
            } else {
                let random = {
                    let mut rng = rand::thread_rng();
                    rng.gen::<u64>()
                };
                set_guild_hash(db, guild_id, random).await
            }
        }
        Err(e) => Err(anyhow!("Error getting guild hash from database: {:?}", e)),
    }
}
