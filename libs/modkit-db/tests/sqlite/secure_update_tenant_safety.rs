#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Integration tests for tenant safety in update paths.
//!
//! Security contract:
//! - No raw SQL in tests.
//! - Schema is created via `sea-orm-migration` definitions executed by the migration runner.

use modkit_db::migration_runner::run_migrations_for_testing;
use modkit_db::secure::{
    Db, DbConn, ScopableEntity, ScopeError, SecureUpdateExt, secure_insert,
    secure_update_with_scope,
};
use modkit_db::{ConnectOpts, connect_db};
use modkit_security::{AccessScope, pep_properties};
use sea_orm::Set;
use sea_orm::entity::prelude::*;
use sea_orm_migration::prelude as mig;
use uuid::Uuid;

mod tenant_ent {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "tenant_update_test")]
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

impl ScopableEntity for tenant_ent::Entity {
    fn tenant_col() -> Option<<Self as EntityTrait>::Column> {
        Some(tenant_ent::Column::TenantId)
    }
    fn resource_col() -> Option<<Self as EntityTrait>::Column> {
        Some(tenant_ent::Column::Id)
    }
    fn owner_col() -> Option<<Self as EntityTrait>::Column> {
        None
    }
    fn type_col() -> Option<<Self as EntityTrait>::Column> {
        None
    }
    fn resolve_property(property: &str) -> Option<<Self as EntityTrait>::Column> {
        match property {
            p if p == pep_properties::OWNER_TENANT_ID => Self::tenant_col(),
            p if p == pep_properties::RESOURCE_ID => Self::resource_col(),
            _ => None,
        }
    }
}

struct CreateSecureUpdateTenantSafetyTables;

impl mig::MigrationName for CreateSecureUpdateTenantSafetyTables {
    fn name(&self) -> &'static str {
        "m001_create_secure_update_tenant_safety_tables"
    }
}

#[async_trait::async_trait]
impl mig::MigrationTrait for CreateSecureUpdateTenantSafetyTables {
    async fn up(&self, manager: &mig::SchemaManager) -> Result<(), mig::DbErr> {
        manager
            .create_table(
                mig::Table::create()
                    .table(mig::Alias::new("tenant_update_test"))
                    .if_not_exists()
                    .col(
                        mig::ColumnDef::new(mig::Alias::new("id"))
                            .uuid()
                            .not_null()
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

        Ok(())
    }

    async fn down(&self, manager: &mig::SchemaManager) -> Result<(), mig::DbErr> {
        manager
            .drop_table(
                mig::Table::drop()
                    .table(mig::Alias::new("tenant_update_test"))
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

// Helper struct to manage test database lifecycle
struct TestDb {
    db: Db,
}

impl TestDb {
    async fn new() -> Self {
        let test_id = Uuid::new_v4();
        let dsn = format!(
            "sqlite:file:memdb_secure_update_tenant_safety_{test_id}?mode=memory&cache=shared"
        );

        let opts = ConnectOpts {
            max_conns: Some(1),
            min_conns: Some(1),
            ..Default::default()
        };

        let db = connect_db(&dsn, opts).await.expect("connect");

        run_migrations_for_testing(&db, vec![Box::new(CreateSecureUpdateTenantSafetyTables)])
            .await
            .expect("migrate");

        Self { db }
    }

    fn conn(&self) -> DbConn<'_> {
        self.db.conn().expect("conn")
    }
}

#[tokio::test]
async fn tenant_scoped_update_allows_row_in_scope_and_no_tenant_change() {
    let test_db = TestDb::new().await;
    let conn = test_db.conn();
    let tenant_a = Uuid::new_v4();
    let scope_a = AccessScope::for_tenant(tenant_a);

    let id = Uuid::new_v4();
    let created = secure_insert::<tenant_ent::Entity>(
        tenant_ent::ActiveModel {
            id: Set(id),
            tenant_id: Set(tenant_a),
            name: Set("before".to_owned()),
        },
        &scope_a,
        &conn,
    )
    .await
    .expect("insert");
    assert_eq!(created.name, "before");

    let updated = secure_update_with_scope::<tenant_ent::Entity>(
        tenant_ent::ActiveModel {
            id: Set(id),
            // tenant_id is intentionally NotSet: allowed (no change).
            name: Set("after".to_owned()),
            ..Default::default()
        },
        &scope_a,
        id,
        &conn,
    )
    .await
    .expect("update");

    assert_eq!(updated.name, "after");
    assert_eq!(updated.tenant_id, tenant_a);
}

#[tokio::test]
async fn tenant_scoped_update_rejects_cross_tenant_update_by_id() {
    let test_db = TestDb::new().await;
    let conn = test_db.conn();
    let tenant_a = Uuid::new_v4();
    let tenant_b = Uuid::new_v4();
    let scope_a = AccessScope::for_tenant(tenant_a);
    let scope_b = AccessScope::for_tenant(tenant_b);

    let id_b = Uuid::new_v4();
    let _ = secure_insert::<tenant_ent::Entity>(
        tenant_ent::ActiveModel {
            id: Set(id_b),
            tenant_id: Set(tenant_b),
            name: Set("row-b".to_owned()),
        },
        &scope_b,
        &conn,
    )
    .await
    .expect("insert");

    let err = secure_update_with_scope::<tenant_ent::Entity>(
        tenant_ent::ActiveModel {
            id: Set(id_b),
            name: Set("should-fail".to_owned()),
            ..Default::default()
        },
        &scope_a,
        id_b,
        &conn,
    )
    .await
    .expect_err("must deny");

    assert!(matches!(err, ScopeError::Denied(_)));
}

#[tokio::test]
async fn tenant_scoped_update_rejects_attempt_to_change_tenant_id() {
    let test_db = TestDb::new().await;
    let conn = test_db.conn();
    let tenant_a = Uuid::new_v4();
    let tenant_b = Uuid::new_v4();
    let scope_a = AccessScope::for_tenant(tenant_a);

    let id = Uuid::new_v4();
    let _ = secure_insert::<tenant_ent::Entity>(
        tenant_ent::ActiveModel {
            id: Set(id),
            tenant_id: Set(tenant_a),
            name: Set("before".to_owned()),
        },
        &scope_a,
        &conn,
    )
    .await
    .expect("insert");

    let err = secure_update_with_scope::<tenant_ent::Entity>(
        tenant_ent::ActiveModel {
            id: Set(id),
            tenant_id: Set(tenant_b),
            name: Set("after".to_owned()),
        },
        &scope_a,
        id,
        &conn,
    )
    .await
    .expect_err("must reject tenant change");

    assert!(matches!(err, ScopeError::Denied("tenant_id is immutable")));
}

#[tokio::test]
async fn update_many_rejects_setting_tenant_id() {
    use sea_orm::sea_query::Expr;

    let test_db = TestDb::new().await;
    let conn = test_db.conn();
    let tenant_a = Uuid::new_v4();
    let tenant_b = Uuid::new_v4();
    let scope_a = AccessScope::for_tenant(tenant_a);

    let id = Uuid::new_v4();
    let _ = secure_insert::<tenant_ent::Entity>(
        tenant_ent::ActiveModel {
            id: Set(id),
            tenant_id: Set(tenant_a),
            name: Set("before".to_owned()),
        },
        &scope_a,
        &conn,
    )
    .await
    .expect("insert");

    // Use the trait-based API instead of SecureConn method
    let err = tenant_ent::Entity::update_many()
        .secure()
        .scope_with(&scope_a)
        .col_expr(tenant_ent::Column::TenantId, Expr::value(tenant_b))
        .filter(sea_orm::Condition::all().add(Expr::col(tenant_ent::Column::Id).eq(id)))
        .exec(&conn)
        .await
        .expect_err("must reject tenant_id update");

    assert!(matches!(err, ScopeError::Denied("tenant_id is immutable")));
}
