#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(feature = "integration")]

//! Integration tests for database connection parameters.
//!
//! # Security
//! These tests must not use raw SQL execution from test code. They are reduced to
//! **connectivity + secure transaction smoke tests**.

mod common;

use anyhow::Result;
use modkit_db::migration_runner::run_migrations_for_testing;
use modkit_db::secure::{ScopableEntity, SecureEntityExt, secure_insert};
use modkit_db::{DbConnConfig, build_db};
use modkit_security::AccessScope;
use sea_orm::Set;
use sea_orm::entity::prelude::*;
use sea_orm_migration::prelude as mig;
use sea_orm_migration::prelude::Iden;
use sea_orm_migration::sea_query;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Iden)]
enum SmokeTbl {
    #[iden = "params_smoke"]
    Table,
    Id,
    TenantId,
    Name,
}

struct CreateParamsSmoke;

impl mig::MigrationName for CreateParamsSmoke {
    #[allow(clippy::unnecessary_literal_bound)]
    fn name(&self) -> &str {
        "m001_create_params_smoke"
    }
}

#[async_trait::async_trait]
impl mig::MigrationTrait for CreateParamsSmoke {
    async fn up(&self, manager: &mig::SchemaManager) -> Result<(), mig::DbErr> {
        manager
            .create_table(
                mig::Table::create()
                    .table(SmokeTbl::Table)
                    .if_not_exists()
                    .col(
                        mig::ColumnDef::new(SmokeTbl::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(mig::ColumnDef::new(SmokeTbl::TenantId).uuid().not_null())
                    .col(mig::ColumnDef::new(SmokeTbl::Name).string().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &mig::SchemaManager) -> Result<(), mig::DbErr> {
        manager
            .drop_table(mig::Table::drop().table(SmokeTbl::Table).to_owned())
            .await
    }
}

mod ent {
    use sea_orm::entity::prelude::*;
    use uuid::Uuid;

    #[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "params_smoke")]
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

impl ScopableEntity for ent::Entity {
    fn tenant_col() -> Option<<Self as EntityTrait>::Column> {
        Some(ent::Column::TenantId)
    }
    fn resource_col() -> Option<<Self as EntityTrait>::Column> {
        Some(ent::Column::Id)
    }
    fn owner_col() -> Option<<Self as EntityTrait>::Column> {
        None
    }
    fn type_col() -> Option<<Self as EntityTrait>::Column> {
        None
    }
}

async fn smoke_secure_tx(db: modkit_db::Db) -> Result<()> {
    run_migrations_for_testing(&db, vec![Box::new(CreateParamsSmoke)])
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    let tenant_id = Uuid::new_v4();
    let scope = AccessScope::tenants_only(vec![tenant_id]);
    let scope_for_tx = scope.clone();
    let id = Uuid::new_v4();

    let (db, res) = db
        .transaction(move |tx| {
            let scope = scope_for_tx.clone();
            Box::pin(async move {
                let am = ent::ActiveModel {
                    id: Set(id),
                    tenant_id: Set(tenant_id),
                    name: Set("ok".to_owned()),
                };
                let _ = secure_insert::<ent::Entity>(am, &scope, tx).await?;
                Ok::<(), anyhow::Error>(())
            })
        })
        .await;
    res?;

    let conn = db.conn().expect("conn");
    let count = ent::Entity::find()
        .secure()
        .scope_with(&scope)
        .count(&conn)
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    assert_eq!(count, 1);

    Ok(())
}

#[cfg(feature = "pg")]
#[tokio::test]
async fn pg_params_smoke() -> Result<()> {
    let dut = common::bring_up_postgres().await?;
    let mut params = HashMap::new();
    params.insert("application_name".to_owned(), "test_app_modkit".to_owned());

    let config = DbConnConfig {
        dsn: Some(dut.url),
        params: Some(params),
        ..Default::default()
    };
    let db = build_db(config, None).await?;
    smoke_secure_tx(db).await?;
    Ok(())
}

#[cfg(feature = "mysql")]
#[tokio::test]
async fn mysql_params_smoke() -> Result<()> {
    let dut = common::bring_up_mysql().await?;
    let mut params = HashMap::new();
    params.insert("connect_timeout".to_owned(), "10".to_owned());

    let config = DbConnConfig {
        dsn: Some(dut.url),
        params: Some(params),
        ..Default::default()
    };
    let db = build_db(config, None).await?;
    smoke_secure_tx(db).await?;
    Ok(())
}

#[cfg(feature = "sqlite")]
#[tokio::test]
async fn sqlite_params_smoke() -> Result<()> {
    let mut params = HashMap::new();
    params.insert("busy_timeout".to_owned(), "5000".to_owned());

    let config = DbConnConfig {
        dsn: Some("sqlite::memory:".to_owned()),
        params: Some(params),
        ..Default::default()
    };
    let db = build_db(config, None).await?;
    smoke_secure_tx(db).await?;
    Ok(())
}
