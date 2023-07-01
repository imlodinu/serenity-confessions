use std::collections::HashMap;

use anyhow::{anyhow, Result};
use migration::OnConflict;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, InsertResult, QueryFilter, Set};
use tracing::info;

use crate::entity::guild_subjects;
use crate::entity::guild_user_subjects;

pub async fn get_guild_subjects_raw(
    db: &DatabaseConnection,
    guild_id: u64,
) -> Result<Vec<guild_subjects::Model>> {
    match guild_subjects::Entity::find()
        .filter(guild_subjects::Column::GuildId.eq(guild_id))
        .all(db)
        .await
    {
        Ok(subjects) => Ok(subjects),
        Err(why) => Err(anyhow!("Error getting subjects from database: {:?}", why)),
    }
}

pub async fn get_guild_subjects(db: &DatabaseConnection, guild_id: u64) -> Result<Vec<String>> {
    match get_guild_subjects_raw(db, guild_id).await {
        Ok(subject_names) => Ok(subject_names
            .iter()
            .map(|s| s.name.clone())
            .collect::<Vec<String>>()),
        Err(why) => Err(why),
    }
}

pub async fn add_guild_subject(
    db: &DatabaseConnection,
    guild_id: u64,
    subject: String,
) -> Result<InsertResult<guild_subjects::ActiveModel>> {
    let this_subject = guild_subjects::ActiveModel {
        guild_id: Set(guild_id),
        name: Set(subject.clone()),
        ..Default::default()
    };
    if let Ok(Some(model)) = guild_has_subject(db, guild_id, subject.clone()).await {
        return Err(anyhow!("Subject already exists in database: {:?}", model));
    }

    let add_result = guild_subjects::Entity::insert(this_subject.clone())
        .exec(db)
        .await;
    match add_result {
        Ok(r) => Ok(r),
        Err(e) => Err(anyhow!("Error adding subject to database: {:?}", e)),
    }
}

pub async fn guild_has_subject(
    db: &DatabaseConnection,
    guild_id: u64,
    subject: String,
) -> Result<Option<guild_subjects::Model>> {
    let found_subject = guild_subjects::Entity::find()
        .filter(guild_subjects::Column::GuildId.eq(guild_id))
        .filter(guild_subjects::Column::Name.eq(subject.clone()))
        .one(db)
        .await;

    match found_subject {
        Ok(Some(m)) => Ok(Some(m)),
        Ok(None) => Ok(None),
        Err(e) => Err(anyhow!("Error getting subject from database: {:?}", e)),
    }
}

pub async fn remove_guild_subject(
    db: &DatabaseConnection,
    guild_id: u64,
    subject: String,
) -> Result<()> {
    if let Ok(Some(model)) = guild_has_subject(db, guild_id, subject.clone()).await {
        let remove_result = guild_subjects::Entity::delete_by_id(model.id)
            .exec(db)
            .await;
        match remove_result {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!("Error removing subject from database: {:?}", e)),
        }
    } else {
        Err(anyhow!("Subject not found in database"))
    }
}

pub async fn get_user_subjects(
    db: &DatabaseConnection,
    guild_id: u64,
    user_id: u64,
) -> Result<Vec<String>> {
    let guild_subjects_res = get_guild_subjects_raw(db, guild_id).await;
    if let Err(why) = guild_subjects_res {
        return Err(why);
    }
    let guild_subjects = guild_subjects_res.unwrap();
    let guild_subject_map: HashMap<_, _> =
        guild_subjects.into_iter().map(|s| (s.id, s.name)).collect();
    match get_user_subjects_raw(db, guild_id, user_id).await {
        Ok(user_subjects) => Ok(user_subjects
            .iter()
            .map(|s| {
                guild_subject_map
                    .get(&s.subject_id)
                    .map(|v| v.to_owned())
                    .unwrap_or("?".into())
                    .clone()
            })
            .collect::<Vec<String>>()),
        Err(why) => Err(anyhow!("Error getting subjects from database: {:?}", why)),
    }
}

pub async fn get_user_subjects_raw(
    db: &DatabaseConnection,
    guild_id: u64,
    user_id: u64,
) -> Result<Vec<guild_user_subjects::Model>> {
    match guild_user_subjects::Entity::find()
        .filter(guild_user_subjects::Column::GuildId.eq(guild_id))
        .filter(guild_user_subjects::Column::UserId.eq(user_id))
        .all(db)
        .await
    {
        Ok(user_subjects) => Ok(user_subjects),
        Err(why) => Err(anyhow!("Error getting subjects from database: {:?}", why)),
    }
}

pub async fn add_user_subjects(
    db: &DatabaseConnection,
    guild_id: u64,
    user_id: u64,
    subjects: Vec<String>,
) -> Result<()> {
    let mut subject_ids = Vec::new();
    for subject in subjects {
        let subject_id = match guild_has_subject(db, guild_id, subject.clone()).await {
            Ok(Some(model)) => model.id,
            Ok(None) => {
                return Err(anyhow!(format!("Subject {} not found in guild", subject)));
            }
            Err(why) => {
                return Err(why);
            }
        };
        subject_ids.push(subject_id);
    }

    // check if already has some
    let user_subjects_res = get_user_subjects_raw(db, guild_id, user_id).await;
    if let Err(why) = user_subjects_res {
        return Err(why);
    }
    let check_user_subjects = user_subjects_res.unwrap();

    let mut user_subjects = Vec::new();
    for subject_id in subject_ids {
        let user_subject = guild_user_subjects::ActiveModel {
            guild_id: Set(guild_id),
            user_id: Set(user_id),
            subject_id: Set(subject_id),
            ..Default::default()
        };
        if check_user_subjects
            .iter()
            .find(|s| s.subject_id == subject_id)
            .is_some()
        {
            continue;
        }
        user_subjects.push(user_subject);
    }
    let add_result = guild_user_subjects::Entity::insert_many(user_subjects)
        .exec(db)
        .await;
    match add_result {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!("Error adding subjects to database: {:?}", e)),
    }
}

pub async fn remove_user_subjects(
    db: &DatabaseConnection,
    guild_id: u64,
    user_id: u64,
    subjects: Vec<String>,
) -> Result<()> {
    let mut subject_ids = Vec::new();
    for subject in subjects {
        let subject_id = match guild_has_subject(db, guild_id, subject.clone()).await {
            Ok(Some(model)) => model.id,
            Ok(None) => {
                return Err(anyhow!(format!("Subject {} not found in guild", subject)));
            }
            Err(why) => {
                return Err(why);
            }
        };
        subject_ids.push(subject_id);
    }
    let remove_result = guild_user_subjects::Entity::delete_many()
        .filter(guild_user_subjects::Column::GuildId.eq(guild_id))
        .filter(guild_user_subjects::Column::UserId.eq(user_id))
        .filter(guild_user_subjects::Column::SubjectId.is_in(subject_ids))
        .exec(db)
        .await;
    match remove_result {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!("Error removing subjects from database: {:?}", e)),
    }
}

pub async fn get_users_with_subject(
    db: &DatabaseConnection,
    guild_id: u64,
    subject: String,
) -> Result<Vec<u64>> {
    let subject_id = match guild_has_subject(db, guild_id, subject.clone()).await {
        Ok(Some(model)) => model.id,
        Ok(None) => {
            return Err(anyhow!(format!("Subject {} not found in guild", subject)));
        }
        Err(why) => {
            return Err(why);
        }
    };
    match guild_user_subjects::Entity::find()
        .filter(guild_user_subjects::Column::GuildId.eq(guild_id))
        .filter(guild_user_subjects::Column::SubjectId.eq(subject_id))
        .all(db)
        .await
    {
        Ok(user_subjects) => Ok(user_subjects
            .iter()
            .map(|s| s.user_id)
            .collect::<Vec<u64>>()),
        Err(why) => Err(anyhow!("Error getting subjects from database: {:?}", why)),
    }
}