pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20230606_141731_change_id_to_u64_snowflake;
mod m20230606_152158_add_channels;
mod m20230608_104157_add_confessions_hashes;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20230606_141731_change_id_to_u64_snowflake::Migration),
            Box::new(m20230606_152158_add_channels::Migration),
            Box::new(m20230608_104157_add_confessions_hashes::Migration),
        ]
    }
}
