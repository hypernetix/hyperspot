#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Transaction tests for the **secure** transaction API.
//!
//! Security contract:
//! - Tests must not use raw SQL execution from test code.
//! - All DB access happens via `SecureConn` / `SecureTx` + secure wrappers.

use modkit_db::migration_runner::run_migrations_for_testing;
use modkit_db::secure::{ScopableEntity, SecureConn, SecureEntityExt, secure_insert};
use modkit_db::{ConnectOpts, DbHandle};
use modkit_security::AccessScope;
use sea_orm::Set;
use sea_orm::entity::prelude::*;
use sea_orm_migration::prelude as mig;
use uuid::Uuid;

mod ent {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "tx_test")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub tenant_id: Uuid,
        pub resource_id: Uuid,
        pub val: String,
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
        Some(ent::Column::ResourceId)
    }

    fn owner_col() -> Option<<Self as EntityTrait>::Column> {
        None
    }

    fn type_col() -> Option<<Self as EntityTrait>::Column> {
        None
    }
}

struct CreateTxTest;

impl mig::MigrationName for CreateTxTest {
    fn name(&self) -> &'static str {
        "m001_create_tx_test"
    }
}

#[async_trait::async_trait]
impl mig::MigrationTrait for CreateTxTest {
    async fn up(&self, manager: &mig::SchemaManager) -> Result<(), mig::DbErr> {
        manager
            .create_table(
                mig::Table::create()
                    .table(mig::Alias::new("tx_test"))
                    .if_not_exists()
                    .col(
                        mig::ColumnDef::new(mig::Alias::new("id"))
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        mig::ColumnDef::new(mig::Alias::new("tenant_id"))
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        mig::ColumnDef::new(mig::Alias::new("resource_id"))
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        mig::ColumnDef::new(mig::Alias::new("val"))
                            .string()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &mig::SchemaManager) -> Result<(), mig::DbErr> {
        manager
            .drop_table(
                mig::Table::drop()
                    .table(mig::Alias::new("tx_test"))
                    .to_owned(),
            )
            .await
    }
}

async fn setup(db: &DbHandle) -> SecureConn {
    run_migrations_for_testing(db, vec![Box::new(CreateTxTest)])
        .await
        .expect("migrate");
    db.sea_secure()
}

#[tokio::test]
async fn sqlite_with_tx_commit_persists_changes() {
    let opts = ConnectOpts {
        max_conns: Some(1),
        ..Default::default()
    };
    let db = DbHandle::connect("sqlite:file:memdb_commit?mode=memory&cache=shared", opts)
        .await
        .expect("Failed to connect to database");
    let conn = setup(&db).await;

    let tenant_id = Uuid::new_v4();
    let scope = AccessScope::tenants_only(vec![tenant_id]);
    let scope_for_tx = scope.clone();
    let resource_id = Uuid::new_v4();

    conn.transaction(move |tx| {
        let scope = scope_for_tx.clone();
        Box::pin(async move {
            let am = ent::ActiveModel {
                tenant_id: Set(tenant_id),
                resource_id: Set(resource_id),
                val: Set("committed".to_owned()),
                ..Default::default()
            };
            let _ = secure_insert::<ent::Entity>(am, &scope, tx).await?;
            Ok::<(), anyhow::Error>(())
        })
    })
    .await
    .expect("Transaction failed");

    let count = ent::Entity::find()
        .secure()
        .scope_with(&scope)
        .count(&conn)
        .await
        .expect("count");
    assert_eq!(count, 1);
}

#[tokio::test]
async fn sqlite_with_tx_error_rolls_back() {
    let opts = ConnectOpts {
        max_conns: Some(1),
        ..Default::default()
    };
    let db = DbHandle::connect("sqlite:file:memdb_rollback?mode=memory&cache=shared", opts)
        .await
        .expect("Failed to connect to database");
    let conn = setup(&db).await;

    let tenant_id = Uuid::new_v4();
    let scope = AccessScope::tenants_only(vec![tenant_id]);
    let scope_for_tx = scope.clone();
    let resource_id = Uuid::new_v4();

    let res: anyhow::Result<()> = conn
        .transaction(move |tx| {
            let scope = scope_for_tx.clone();
            Box::pin(async move {
                let am = ent::ActiveModel {
                    tenant_id: Set(tenant_id),
                    resource_id: Set(resource_id),
                    val: Set("should_rollback".to_owned()),
                    ..Default::default()
                };
                let _ = secure_insert::<ent::Entity>(am, &scope, tx).await?;
                anyhow::bail!("Simulated error");
            })
        })
        .await;

    assert!(res.is_err());

    let count = ent::Entity::find()
        .secure()
        .scope_with(&scope)
        .count(&conn)
        .await
        .expect("count");
    assert_eq!(count, 0);
}

#[tokio::test]
async fn sqlite_with_tx_returns_value() {
    let opts = ConnectOpts {
        max_conns: Some(1),
        ..Default::default()
    };
    let db = DbHandle::connect("sqlite:file:memdb_returns?mode=memory&cache=shared", opts)
        .await
        .expect("Failed to connect to database");
    let conn = setup(&db).await;

    let tenant_id = Uuid::new_v4();
    let scope = AccessScope::tenants_only(vec![tenant_id]);
    let resource_id = Uuid::new_v4();

    let inserted_id: Uuid = conn
        .transaction(move |tx| {
            let scope = scope.clone();
            Box::pin(async move {
                let am = ent::ActiveModel {
                    tenant_id: Set(tenant_id),
                    resource_id: Set(resource_id),
                    val: Set("test_value".to_owned()),
                    ..Default::default()
                };
                let _ = secure_insert::<ent::Entity>(am, &scope, tx).await?;
                Ok::<Uuid, anyhow::Error>(resource_id)
            })
        })
        .await
        .expect("Transaction failed");

    assert_eq!(inserted_id, resource_id);

    let found = ent::Entity::find()
        .secure()
        .scope_with(&AccessScope::both(vec![tenant_id], vec![resource_id]))
        .one(&conn)
        .await
        .expect("select")
        .expect("row must exist");
    assert_eq!(found.val, "test_value");
}
