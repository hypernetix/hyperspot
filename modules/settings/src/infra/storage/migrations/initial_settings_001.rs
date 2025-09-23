use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Settings::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Settings::UserID).uuid().not_null())
                    .col(ColumnDef::new(Settings::TenantID).uuid().not_null())
                    .col(ColumnDef::new(Settings::Theme).string().not_null())
                    .col(ColumnDef::new(Settings::Language).string().not_null())
                    .primary_key(
                        Index::create()
                            .col(Settings::UserID)
                            .col(Settings::TenantID),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Settings::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Settings {
    Table,
    UserID,
    TenantID,
    Theme,
    Language,
}
