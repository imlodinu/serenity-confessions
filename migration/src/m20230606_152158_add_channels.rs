use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Channels::Table)
                    .col(
                        ColumnDef::new(Channels::Id)
                            .big_unsigned()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Channels::GuildId).big_unsigned().not_null())
                    .col(ColumnDef::new(Channels::ChannelUse).integer().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Guild::Table)
                    .modify_column(ColumnDef::new(Guild::Id).big_unsigned().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Channels::Table).to_owned())
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Guild::Table)
                    .modify_column(ColumnDef::new(Guild::Id).big_unsigned())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Channels {
    Table,
    Id,
    GuildId,
    ChannelUse,
}

#[derive(Iden)]
enum Guild {
    Table,
    Id,
}
