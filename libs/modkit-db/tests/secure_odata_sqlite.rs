#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(feature = "sqlite")]

//! `SQLite` integration tests for `OData` + Secure ORM execution.
//!
//! Security contract:
//! - Do not use any raw SeaORM/SQLx executors from test code.
//! - Execute queries only through `SecureConn` / `SecureTx` + secure wrappers.

use anyhow::anyhow;
use modkit_db::migration_runner::run_migrations_for_testing;
use modkit_db::odata::FieldMap;
use modkit_db::odata::pager::OPager;
use modkit_db::secure::{Db, DbConn, ScopableEntity, secure_insert};
use modkit_db::{ConnectOpts, connect_db};
use modkit_odata::ODataQuery;
use modkit_odata::filter::FieldKind;
use modkit_security::AccessScope;
use sea_orm::Set;
use sea_orm::entity::prelude::*;
use sea_orm_migration::prelude as mig;
use uuid::Uuid;

mod ent {
    use sea_orm::entity::prelude::*;
    use uuid::Uuid;

    #[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "secure_odata_test")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub tenant_id: Uuid,
        pub name: String,
        pub score: i64,
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
        None
    }
    fn owner_col() -> Option<<Self as EntityTrait>::Column> {
        None
    }
    fn type_col() -> Option<<Self as EntityTrait>::Column> {
        None
    }
}

struct CreateSecureOdataTest;

impl mig::MigrationName for CreateSecureOdataTest {
    fn name(&self) -> &'static str {
        "m001_create_secure_odata_test"
    }
}

#[async_trait::async_trait]
impl mig::MigrationTrait for CreateSecureOdataTest {
    async fn up(&self, manager: &mig::SchemaManager) -> Result<(), mig::DbErr> {
        manager
            .create_table(
                mig::Table::create()
                    .table(mig::Alias::new("secure_odata_test"))
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
                    .col(
                        mig::ColumnDef::new(mig::Alias::new("score"))
                            .big_integer()
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
                    .table(mig::Alias::new("secure_odata_test"))
                    .to_owned(),
            )
            .await
    }
}

// Helper struct to manage test database lifecycle
struct TestDb {
    db: Db,
    tenant_id: Uuid,
    scope: AccessScope,
}

impl TestDb {
    async fn new() -> Self {
        let db = connect_db("sqlite::memory:", ConnectOpts::default())
            .await
            .expect("db connect");

        run_migrations_for_testing(&db, vec![Box::new(CreateSecureOdataTest)])
            .await
            .map_err(|e| anyhow!(e.to_string()))
            .expect("migrate");

        let tenant_id = Uuid::new_v4();
        let scope = AccessScope::tenants_only(vec![tenant_id]);

        Self {
            db,
            tenant_id,
            scope,
        }
    }

    fn conn(&self) -> DbConn<'_> {
        self.db.conn().expect("conn")
    }
}

async fn seed<R: modkit_db::secure::DBRunner>(runner: &R, tenant_id: Uuid, scope: &AccessScope) {
    let rows = [("alice", 10), ("bob", 20), ("charlie", 30), ("dave", 40)];

    for (name, score) in rows {
        let am = ent::ActiveModel {
            tenant_id: Set(tenant_id),
            name: Set(name.to_owned()),
            score: Set(score),
            ..Default::default()
        };
        secure_insert::<ent::Entity>(am, scope, runner)
            .await
            .expect("insert");
    }
}

#[tokio::test]
async fn paginate_odata_works_with_secure_conn() {
    let test_db = TestDb::new().await;
    let conn = test_db.conn();
    seed(&conn, test_db.tenant_id, &test_db.scope).await;

    let fmap: FieldMap<ent::Entity> = FieldMap::new()
        .insert_with_extractor("id", ent::Column::Id, FieldKind::I64, |m: &ent::Model| {
            m.id.to_string()
        })
        .insert("name", ent::Column::Name, FieldKind::String)
        .insert("score", ent::Column::Score, FieldKind::I64);

    let q = ODataQuery {
        limit: Some(2),
        ..Default::default()
    };

    let page = OPager::<ent::Entity, _>::new(&test_db.scope, &conn, &fmap)
        .fetch(&q, |m| (m.name, m.score))
        .await
        .expect("fetch");

    assert_eq!(page.items.len(), 2, "page size");
}
