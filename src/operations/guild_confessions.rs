use anyhow::{anyhow, Result};
use rand::Rng;
use sea_orm::{DatabaseConnection, EntityTrait, Set, ColumnTrait};

use crate::entity::guild_confessions;

pub async fn set_guild_confessions(
    db: &DatabaseConnection,
    guild: guild_confessions::Model,
) -> Result<()> {
    let guild_hash = guild_confessions::ActiveModel {
        guild_id: Set(guild.guild_id),
        hash: Set(guild.hash),
        lock_shuffle: Set(guild.lock_shuffle),
    };

    let guild_confession_result = guild_confessions::Entity::update(guild_hash)
        .exec(db)
        .await;

    match guild_confession_result {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(
            "Error inserting guild confessions into database: {:?}",
            e
        )),
    }
}

pub async fn get_or_new_guild_confessions(
    db: &DatabaseConnection,
    guild_id: u64,
) -> Result<guild_confessions::Model> {
    let guild_confession_result = guild_confessions::Entity::find_by_id(guild_id)
        .one(db)
        .await;

    match guild_confession_result {
        Ok(guild_confession_opt) => {
            if let Some(guild_confession) = guild_confession_opt {
                Ok(guild_confession)
            } else {
                let random = {
                    let mut rng = rand::thread_rng();
                    rng.gen::<u32>()
                };
                let model = guild_confessions::Model {
                    guild_id,
                    hash: random as u64,
                    lock_shuffle: false as i8,
                };
                match set_guild_confessions(db, model.clone()).await {
                    Ok(_) => Ok(model),
                    Err(why) => Err(anyhow!(
                        "Error setting guild confessions in database: {:?}",
                        why
                    )),
                }
            }
        }
        Err(e) => Err(anyhow!(
            "Error getting guild confessions from database: {:?}",
            e
        )),
    }
}

pub async fn shuffle_guild_hash(
    db: &DatabaseConnection,
    guild_id: u64,
) -> Result<guild_confessions::Model> {
    let random = {
        let mut rng = rand::thread_rng();
        rng.gen::<u32>().max(u32::MAX)
    };
    let guild_res = get_or_new_guild_confessions(db, guild_id).await;
    if let Err(why) = guild_res {
        return Err(anyhow!(
            "Error getting guild confessions from database: {:?}",
            why
        ));
    }
    let mut guild = guild_res.unwrap();
    guild.hash = random as u64;
    match set_guild_confessions(db, guild.clone()).await {
        Ok(_) => Ok(guild),
        Err(why) => Err(anyhow!(
            "Error setting guild confessions in database: {:?}",
            why
        )),
    }
}

pub async fn set_guild_shuffle_lock(
    db: &DatabaseConnection,
    guild_id: u64,
    lock_shuffle: bool,
) -> Result<guild_confessions::Model> {
    let guild_res = get_or_new_guild_confessions(db, guild_id).await;
    if let Err(why) = guild_res {
        return Err(anyhow!(
            "Error getting guild confessions from database: {:?}",
            why
        ));
    }
    let mut guild = guild_res.unwrap();
    guild.lock_shuffle = if lock_shuffle { 1 } else { 0 };
    match set_guild_confessions(db, guild.clone()).await {
        Ok(_) => Ok(guild),
        Err(why) => Err(anyhow!(
            "Error setting guild confessions in database: {:?}",
            why
        )),
    }
}

pub async fn get_guild_shuffle_lock(db: &DatabaseConnection, guild_id: u64) -> Result<bool> {
    let guild_res = get_or_new_guild_confessions(db, guild_id).await;
    if let Err(why) = guild_res {
        return Err(anyhow!(
            "Error getting guild confessions from database: {:?}",
            why
        ));
    }
    let guild = guild_res.unwrap();
    Ok(guild.lock_shuffle == 1)
}
