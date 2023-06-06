use anyhow::{Result, anyhow};
use migration::OnConflict;
use sea_orm::{DatabaseConnection, EntityTrait, Set, InsertResult, ColumnTrait, QueryFilter};
use tracing::warn;

use crate::entity::channels;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, poise::ChoiceParameter)]
pub enum ChannelUse {
    #[name = "none"]
    None,
    #[name = "confession"]
    ConfessionOut,
    #[name = "vetting"]
    ConfessionVet,
}

pub async fn get_channels(db: &DatabaseConnection) -> Result<Vec<channels::Model>> {
    let channels = channels::Entity::find().all(db).await;
    match channels {
        Ok(channels) => Ok(channels),
        Err(e) => Err(anyhow!("Error getting channels from database: **{:?}**", e)),
    }
}

pub async fn get_channels_in_guild(db: &DatabaseConnection, guild_id: u64) -> Result<Vec<channels::Model>> {
    let found_channels = channels::Entity::find()
        .filter(channels::Column::GuildId.eq(guild_id))
        .all(db)
        .await;
    match found_channels {
        Ok(channels) => Ok(channels),
        Err(e) => Err(anyhow!("Error getting channels from database: **{:?}**", e)),
    }
}

pub async fn get_channels_in_guild_with_use(db: &DatabaseConnection, guild_id: u64, channel_use: ChannelUse) -> Result<Vec<channels::Model>> {
    let found_channels = channels::Entity::find()
        .filter(channels::Column::GuildId.eq(guild_id))
        .filter(channels::Column::ChannelUse.eq(channel_use as i32))
        .all(db)
        .await;
    match found_channels {
        Ok(channels) => Ok(channels),
        Err(e) => Err(anyhow!("Error getting channels from database: **{:?}**", e)),
    }
}

pub async fn add_channel_for_guild(db: &DatabaseConnection, guild_id: u64, channel_id: u64, channel_use: ChannelUse) -> Result<InsertResult<channels::ActiveModel>> {
    let this_channel = channels::ActiveModel {
        id: Set(channel_id),
        guild_id: Set(guild_id),
        channel_use: Set(channel_use as i32),
    };
    let add_result = channels::Entity::insert(this_channel)
        .on_conflict(
            OnConflict::column(channels::Column::ChannelUse)
                .update_column(channels::Column::ChannelUse)
                .to_owned(),
        )
        .exec(db)
        .await;
    match add_result {
        Ok(_) => Ok(add_result.unwrap()),
        Err(e) => Err(anyhow!("Error adding channel to database: **{:?}**", e)),
    }
}