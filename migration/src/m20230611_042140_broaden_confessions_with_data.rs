use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .rename_table(
                Table::rename()
                    .table(ConfessionGuildHashes::Table, GuildConfessions::Table)
                    .to_owned(),
            )
            .await?;

        manager.alter_table(
            Table::alter()
                .table(GuildConfessions::Table)
                .add_column(
                    ColumnDef::new(GuildConfessions::LockShuffle)
                        .boolean()
                        .not_null()
                        .default(false),
                )
                .to_owned(),
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .rename_table(
                Table::rename()
                    .table(GuildConfessions::Table, ConfessionGuildHashes::Table)
                    .to_owned(),
            )
            .await?;

        manager.alter_table(
            Table::alter()
                .table(ConfessionGuildHashes::Table)
                .drop_column(GuildConfessions::LockShuffle)
                .to_owned(),
        ).await?;

        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Iden)]
enum ConfessionGuildHashes {
    Table,
    GuildId,
    Hash,
}

#[allow(dead_code)]
#[derive(Iden)]
enum GuildConfessions {
    Table,
    GuildId,
    Hash,
    LockShuffle,
}
