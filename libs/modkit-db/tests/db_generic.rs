#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(feature = "integration")]

mod common;
use anyhow::Result;
use modkit_db::secure::SecureEntityExt;
use sea_orm::EntityTrait;
use sea_orm_migration::prelude as mig;
use sea_orm_migration::prelude::Iden;
use sea_orm_migration::sea_query;

// Create a simple table via migrations (runtime-privileged path).
#[derive(Iden)]
enum GenericTbl {
    #[iden = "test_generic"]
    Table,
    Id,
    TenantId,
    Name,
}

struct CreateGeneric;
impl mig::MigrationName for CreateGeneric {
    #[allow(clippy::unnecessary_literal_bound)]
    fn name(&self) -> &str {
        "m001_create_test_generic"
    }
}
#[async_trait::async_trait]
impl mig::MigrationTrait for CreateGeneric {
    async fn up(&self, manager: &mig::SchemaManager) -> Result<(), mig::DbErr> {
        manager
            .create_table(
                mig::Table::create()
                    .table(GenericTbl::Table)
                    .if_not_exists()
                    .col(
                        mig::ColumnDef::new(GenericTbl::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(mig::ColumnDef::new(GenericTbl::TenantId).uuid().not_null())
                    .col(mig::ColumnDef::new(GenericTbl::Name).string().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &mig::SchemaManager) -> Result<(), mig::DbErr> {
        manager
            .drop_table(mig::Table::drop().table(GenericTbl::Table).to_owned())
            .await
    }
}

// Define a minimal entity mapping for secure query execution.
mod ent {
    use sea_orm::entity::prelude::*;
    use uuid::Uuid;

    #[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "test_generic")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: Uuid,
        pub tenant_id: Uuid,
        pub name: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

impl modkit_db::secure::ScopableEntity for ent::Entity {
    fn tenant_col() -> Option<<Self as sea_orm::EntityTrait>::Column> {
        Some(ent::Column::TenantId)
    }
    fn resource_col() -> Option<<Self as sea_orm::EntityTrait>::Column> {
        Some(ent::Column::Id)
    }
    fn owner_col() -> Option<<Self as sea_orm::EntityTrait>::Column> {
        None
    }
    fn type_col() -> Option<<Self as sea_orm::EntityTrait>::Column> {
        None
    }
}

#[cfg(feature = "sqlite")]
#[tokio::test]
async fn generic_sqlite() -> Result<()> {
    let dut = common::bring_up_sqlite();
    run_common_suite(&dut.url).await
}

#[cfg(feature = "pg")]
#[tokio::test]
async fn generic_postgres() -> Result<()> {
    let dut = common::bring_up_postgres().await?;
    run_common_suite(&dut.url).await
}

#[cfg(feature = "mysql")]
#[tokio::test]
async fn generic_mysql() -> Result<()> {
    let dut = common::bring_up_mysql().await?;
    run_common_suite(&dut.url).await
}

/// Runs the same assertions for any backend.
async fn run_common_suite(database_url: &str) -> Result<()> {
    // Test basic connection
    let db = modkit_db::connect_db(database_url, modkit_db::ConnectOpts::default()).await?;

    // Test DSN redaction (should not panic)
    let redacted = modkit_db::redact_credentials_in_dsn(Some(database_url));
    assert!(!redacted.contains("pass"));

    // Test basic transaction + ORM ops using the secure API (no raw SQLx/SeaORM connections).
    modkit_db::migration_runner::run_migrations_for_testing(&db, vec![Box::new(CreateGeneric)])
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    // Use the secure Db API
    let secure_db = db;

    let tenant_id = uuid::Uuid::new_v4();
    let scope = modkit_security::AccessScope::tenants_only(vec![tenant_id]);
    let scope_for_tx = scope.clone();
    let id = uuid::Uuid::new_v4();

    let (secure_db, res) = secure_db
        .transaction(move |tx| {
            let scope = scope_for_tx.clone();
            Box::pin(async move {
                let am = ent::ActiveModel {
                    id: sea_orm::Set(id),
                    tenant_id: sea_orm::Set(tenant_id),
                    name: sea_orm::Set("test_user".to_owned()),
                };
                let _ = modkit_db::secure::secure_insert::<ent::Entity>(am, &scope, tx).await?;
                Ok::<(), anyhow::Error>(())
            })
        })
        .await;
    res?;

    let conn = secure_db
        .conn()
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let count = ent::Entity::find()
        .secure()
        .scope_with(&scope)
        .count(&conn)
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    assert_eq!(count, 1);

    Ok(())
}
