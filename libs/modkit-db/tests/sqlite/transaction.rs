#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Transaction tests for the **secure** transaction API.
//!
//! Security contract:
//! - Tests must not use raw SQL execution from test code.
//! - All DB access happens via `Db` / `DbConn` / `DbTx` + secure wrappers.
//!
//! This test demonstrates the new secure transaction API that prevents
//! the factory-based bypass vulnerability.

use modkit_db::migration_runner::run_migrations_for_testing;
use modkit_db::secure::{Db, ScopableEntity, SecureEntityExt, secure_insert};
use modkit_db::{ConnectOpts, DbError, connect_db};
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

async fn setup(db: Db) -> Db {
    run_migrations_for_testing(&db, vec![Box::new(CreateTxTest)])
        .await
        .expect("migrate");
    db
}

#[tokio::test]
async fn sqlite_with_tx_commit_persists_changes() {
    let opts = ConnectOpts {
        max_conns: Some(1),
        ..Default::default()
    };
    let db = connect_db("sqlite:file:memdb_commit?mode=memory&cache=shared", opts)
        .await
        .expect("Failed to connect to database");
    let db = setup(db).await;

    let tenant_id = Uuid::new_v4();
    let scope = AccessScope::tenants_only(vec![tenant_id]);
    let scope_for_tx = scope.clone();
    let resource_id = Uuid::new_v4();

    // Transaction consumes db and returns it after completion
    let (db, result) = db
        .transaction(move |tx| {
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
        .await;
    result.expect("Transaction failed");

    // After transaction, use conn() to query
    let conn = db.conn().expect("conn");
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
    let db = connect_db("sqlite:file:memdb_rollback?mode=memory&cache=shared", opts)
        .await
        .expect("Failed to connect to database");
    let db = setup(db).await;

    let tenant_id = Uuid::new_v4();
    let scope = AccessScope::tenants_only(vec![tenant_id]);
    let scope_for_tx = scope.clone();
    let resource_id = Uuid::new_v4();

    let (db, res): (_, anyhow::Result<()>) = db
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

    // Verify rollback: count should be 0
    let conn = db.conn().expect("conn");
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
    let db = connect_db("sqlite:file:memdb_returns?mode=memory&cache=shared", opts)
        .await
        .expect("Failed to connect to database");
    let db = setup(db).await;

    let tenant_id = Uuid::new_v4();
    let scope = AccessScope::tenants_only(vec![tenant_id]);
    let resource_id = Uuid::new_v4();

    let (db, inserted_id) = db
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
        .await;
    let inserted_id = inserted_id.expect("Transaction failed");

    assert_eq!(inserted_id, resource_id);

    let conn = db.conn().expect("conn");
    let found = ent::Entity::find()
        .secure()
        .scope_with(&AccessScope::both(vec![tenant_id], vec![resource_id]))
        .one(&conn)
        .await
        .expect("select")
        .expect("row must exist");
    assert_eq!(found.val, "test_value");
}

/// Test: Task-local guard prevents `conn()` from being called inside a transaction.
///
/// Security: even if code uses another `Db` value inside a transaction closure,
/// calling `conn()` on it must fail. This prevents bypassing transaction isolation
/// by creating non-transactional runners inside a transaction.
#[tokio::test]
async fn sqlite_task_local_guard_prevents_conn_in_tx() {
    let opts = ConnectOpts {
        max_conns: Some(5), // Allow multiple connections
        ..Default::default()
    };
    let db = connect_db("sqlite:file:memdb_guard?mode=memory&cache=shared", opts)
        .await
        .expect("Failed to connect to database");

    // Clone a second `Db` value to attempt bypass inside the transaction.
    let db_for_tx = db.clone();

    let (_, result): (_, anyhow::Result<()>) = db
        .transaction(move |_tx| {
            Box::pin(async move {
                // Attempt to get a non-transactional connection - this MUST fail
                let err = db_for_tx
                    .conn()
                    .expect_err("conn() should fail inside transaction");

                // Verify it's the correct error type
                assert!(
                    matches!(err, DbError::ConnRequestedInsideTx),
                    "Expected ConnRequestedInsideTx, got: {err:?}"
                );

                Ok(())
            })
        })
        .await;

    result.expect("Transaction body should complete");
}

/// Test: `conn()` succeeds outside of transactions.
///
/// Verifies that the task-local guard only blocks `conn()` inside transactions,
/// not outside of them.
#[tokio::test]
async fn sqlite_conn_succeeds_outside_transaction() {
    let opts = ConnectOpts {
        max_conns: Some(1),
        ..Default::default()
    };
    let db = connect_db("sqlite:file:memdb_outside?mode=memory&cache=shared", opts)
        .await
        .expect("Failed to connect to database");

    // conn() should succeed outside of any transaction
    let conn = db.conn();
    assert!(conn.is_ok(), "conn() should succeed outside transaction");
}
