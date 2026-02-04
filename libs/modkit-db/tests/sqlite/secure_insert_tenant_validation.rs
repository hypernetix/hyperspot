#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Integration tests for tenant validation in `secure_insert`.
//!
//! Security contract:
//! - No raw SQL in tests.
//! - Schema is created via `sea-orm-migration` definitions executed by the migration runner.

use modkit_db::migration_runner::run_migrations_for_testing;
use modkit_db::secure::{Db, DbConn, ScopableEntity, ScopeError, secure_insert};
use modkit_db::{ConnectOpts, connect_db};
use modkit_security::AccessScope;
use sea_orm::Set;
use sea_orm::entity::prelude::*;
use sea_orm_migration::prelude as mig;
use uuid::Uuid;

mod tenant_ent {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "tenant_insert_test")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub tenant_id: Uuid,
        pub name: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

impl ScopableEntity for tenant_ent::Entity {
    fn tenant_col() -> Option<<Self as EntityTrait>::Column> {
        Some(tenant_ent::Column::TenantId)
    }
    fn resource_col() -> Option<<Self as EntityTrait>::Column> {
        None
    }
    fn owner_col() -> Option<<Self as EntityTrait>::Column> {
        None
    }
    fn type_col() -> Option<<Self as EntityTrait>::Column> {
        None
    }
}

mod unrestricted_ent {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "unrestricted_insert_test")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub name: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

impl ScopableEntity for unrestricted_ent::Entity {
    fn tenant_col() -> Option<<Self as EntityTrait>::Column> {
        None
    }
    fn resource_col() -> Option<<Self as EntityTrait>::Column> {
        None
    }
    fn owner_col() -> Option<<Self as EntityTrait>::Column> {
        None
    }
    fn type_col() -> Option<<Self as EntityTrait>::Column> {
        None
    }
}

struct CreateSecureInsertTenantValidationTables;

impl mig::MigrationName for CreateSecureInsertTenantValidationTables {
    fn name(&self) -> &'static str {
        "m001_create_secure_insert_tenant_validation_tables"
    }
}

#[async_trait::async_trait]
impl mig::MigrationTrait for CreateSecureInsertTenantValidationTables {
    async fn up(&self, manager: &mig::SchemaManager) -> Result<(), mig::DbErr> {
        manager
            .create_table(
                mig::Table::create()
                    .table(mig::Alias::new("tenant_insert_test"))
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
                        mig::ColumnDef::new(mig::Alias::new("name"))
                            .string()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                mig::Table::create()
                    .table(mig::Alias::new("unrestricted_insert_test"))
                    .if_not_exists()
                    .col(
                        mig::ColumnDef::new(mig::Alias::new("id"))
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        mig::ColumnDef::new(mig::Alias::new("name"))
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
                    .table(mig::Alias::new("tenant_insert_test"))
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                mig::Table::drop()
                    .table(mig::Alias::new("unrestricted_insert_test"))
                    .to_owned(),
            )
            .await
    }
}

// Helper struct to manage test database lifecycle
struct TestDb {
    db: Db,
}

impl TestDb {
    async fn new() -> Self {
        let opts = ConnectOpts {
            max_conns: Some(1),
            min_conns: Some(1),
            ..Default::default()
        };
        let test_id = Uuid::new_v4();
        let dsn = format!(
            "sqlite:file:memdb_secure_insert_tenant_validation_{test_id}?mode=memory&cache=shared"
        );
        let db = connect_db(&dsn, opts).await.expect("db connect");

        run_migrations_for_testing(
            &db,
            vec![Box::new(CreateSecureInsertTenantValidationTables)],
        )
        .await
        .expect("migrate");

        Self { db }
    }

    fn conn(&self) -> DbConn<'_> {
        self.db.conn().expect("conn")
    }
}

async fn setup() -> TestDb {
    TestDb::new().await
}

#[tokio::test]
async fn tenant_scoped_insert_allows_tenant_in_scope() {
    let test_db = setup().await;
    let conn = test_db.conn();
    let tenant_a = Uuid::new_v4();
    let scope = AccessScope::tenants_only(vec![tenant_a]);

    let am = tenant_ent::ActiveModel {
        tenant_id: Set(tenant_a),
        name: Set("ok".to_owned()),
        ..Default::default()
    };

    let _ = secure_insert::<tenant_ent::Entity>(am, &scope, &conn)
        .await
        .expect("insert ok");
}

#[tokio::test]
async fn tenant_scoped_insert_rejects_tenant_not_in_scope() {
    let test_db = setup().await;
    let conn = test_db.conn();
    let tenant_a = Uuid::new_v4();
    let tenant_b = Uuid::new_v4();
    let scope = AccessScope::tenants_only(vec![tenant_a]);

    let am = tenant_ent::ActiveModel {
        tenant_id: Set(tenant_b),
        name: Set("nope".to_owned()),
        ..Default::default()
    };

    let err = secure_insert::<tenant_ent::Entity>(am, &scope, &conn)
        .await
        .expect_err("must be rejected");

    match err {
        ScopeError::TenantNotInScope { tenant_id } => assert_eq!(tenant_id, tenant_b),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[tokio::test]
async fn tenant_scoped_insert_rejects_scope_without_tenants() {
    let test_db = setup().await;
    let conn = test_db.conn();
    let tenant_a = Uuid::new_v4();
    let scope = AccessScope::default(); // deny-all (no tenants)

    let am = tenant_ent::ActiveModel {
        tenant_id: Set(tenant_a),
        name: Set("nope".to_owned()),
        ..Default::default()
    };

    let err = secure_insert::<tenant_ent::Entity>(am, &scope, &conn)
        .await
        .expect_err("must be rejected");

    match err {
        ScopeError::Denied(_) => {}
        other => panic!("unexpected error: {other:?}"),
    }
}

#[tokio::test]
async fn tenant_scoped_insert_requires_tenant_id_in_active_model() {
    let test_db = setup().await;
    let conn = test_db.conn();
    let tenant_a = Uuid::new_v4();
    let scope = AccessScope::tenants_only(vec![tenant_a]);

    let am = tenant_ent::ActiveModel {
        // tenant_id intentionally not set
        name: Set("missing".to_owned()),
        ..Default::default()
    };

    let err = secure_insert::<tenant_ent::Entity>(am, &scope, &conn)
        .await
        .expect_err("must be rejected");

    match err {
        ScopeError::Invalid(msg) => assert_eq!(msg, "tenant_id is required"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[tokio::test]
async fn unrestricted_entity_insert_does_not_require_tenant_scope_or_tenant_id() {
    let test_db = setup().await;
    let conn = test_db.conn();
    let scope = AccessScope::default(); // deny-all for reads, but insert allowed for unrestricted

    let am = unrestricted_ent::ActiveModel {
        name: Set("ok".to_owned()),
        ..Default::default()
    };

    let _ = secure_insert::<unrestricted_ent::Entity>(am, &scope, &conn)
        .await
        .expect("insert ok");
}
