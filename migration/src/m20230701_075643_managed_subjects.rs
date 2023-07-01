use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GuildSubjects::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GuildSubjects::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(GuildSubjects::GuildId)
                            .big_unsigned()
                            .not_null(),
                    )
                    .col(ColumnDef::new(GuildSubjects::Name).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(GuildUserSubjects::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GuildUserSubjects::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(GuildUserSubjects::GuildId)
                            .big_unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GuildUserSubjects::UserId)
                            .big_unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GuildUserSubjects::SubjectId)
                            .integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GuildSubjects::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(GuildUserSubjects::Table).to_owned())
            .await?;

        Ok(())
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum GuildSubjects {
    Table,
    Id,
    GuildId,
    Name,
}

#[derive(Iden)]
enum GuildUserSubjects {
    Table,
    Id,
    GuildId,
    UserId,
    SubjectId,
}
