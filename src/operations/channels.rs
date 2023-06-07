use anyhow::{anyhow, Result};
use migration::OnConflict;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, InsertResult, QueryFilter, Set};

use crate::entity::channels;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, poise::ChoiceParameter)]
pub enum ChannelUse {
    #[name = "none"]
    None,
    #[name = "confession"]
    ConfessionOut,
    #[name = "vetting"]
    ConfessionVet,
}

impl Into<i32> for ChannelUse {
    fn into(self) -> i32 {
        match self {
            ChannelUse::None => 0,
            ChannelUse::ConfessionOut => 1,
            ChannelUse::ConfessionVet => 2,
        }
    }
}

impl From<i32> for ChannelUse {
    fn from(i: i32) -> Self {
        match i {
            0 => ChannelUse::None,
            1 => ChannelUse::ConfessionOut,
            2 => ChannelUse::ConfessionVet,
            _ => ChannelUse::None,
        }
    }
}

impl Into<sea_orm::Value> for ChannelUse {
    fn into(self) -> sea_orm::Value {
        sea_orm::Value::Int(Some(match self {
            ChannelUse::None => 0,
            ChannelUse::ConfessionOut => 1,
            ChannelUse::ConfessionVet => 2,
        }))
    }
}

pub async fn get_channels(db: &DatabaseConnection) -> Result<Vec<channels::Model>> {
    let channels = channels::Entity::find().all(db).await;
    match channels {
        Ok(channels) => Ok(channels),
        Err(e) => Err(anyhow!("Error getting channels from database: {:?}", e)),
    }
}

pub async fn get_channels_in_guild(
    db: &DatabaseConnection,
    guild_id: u64,
) -> Result<Vec<channels::Model>> {
    let found_channels = channels::Entity::find()
        .filter(channels::Column::GuildId.eq(guild_id))
        .all(db)
        .await;
    match found_channels {
        Ok(channels) => Ok(channels),
        Err(e) => Err(anyhow!("Error getting channels from database: {:?}", e)),
    }
}

pub async fn get_channels_in_guild_with_use(
    db: &DatabaseConnection,
    guild_id: u64,
    channel_use: ChannelUse,
) -> Result<Vec<channels::Model>> {
    let found_channels = channels::Entity::find()
        .filter(channels::Column::GuildId.eq(guild_id))
        .filter(channels::Column::ChannelUse.eq(channel_use))
        .all(db)
        .await;
    match found_channels {
        Ok(channels) => Ok(channels),
        Err(e) => Err(anyhow!("Error getting channels from database: {:?}", e)),
    }
}

pub async fn get_channel_use(
    db: &DatabaseConnection,
    guild_id: u64,
    channel_id: u64,
) -> Result<ChannelUse> {
    let found_channel = channels::Entity::find_by_id(channel_id)
        .filter(channels::Column::GuildId.eq(guild_id))
        .one(db)
        .await;
    match found_channel {
        Ok(channel) => match channel {
            Some(channel) => Ok(channel.channel_use.into()),
            None => Ok(ChannelUse::None),
        },
        Err(e) => Err(anyhow!("Error getting channel from database: {:?}", e)),
    }
}

pub async fn add_channel_for_guild(
    db: &DatabaseConnection,
    guild_id: u64,
    channel_id: u64,
    channel_use: ChannelUse,
) -> Result<InsertResult<channels::ActiveModel>> {
    let this_channel = channels::ActiveModel {
        id: Set(channel_id),
        guild_id: Set(guild_id),
        channel_use: Set(channel_use.into()),
    };
    let add_result = channels::Entity::insert(this_channel.clone())
        .on_conflict(
            OnConflict::column(channels::Column::ChannelUse)
                .update_column(channels::Column::ChannelUse)
                .to_owned(),
        )
        .exec(db)
        .await;
    match add_result {
        Ok(result) => Ok(result),
        Err(e) => Err(anyhow!("Error adding channel to database: {:?}", e)),
    }
}
