use super::support::setup_sqlite_db;
use crate::odata::pager::OPager;
use crate::odata::{FieldKind, FieldMap};
use crate::secure::ScopableEntity;
use crate::Result;
use anyhow::anyhow;
use modkit_odata::{ODataOrderBy, ODataQuery, OrderKey, SortDir};
use modkit_security::AccessScope;
use sea_orm::{ConnectionTrait, DatabaseConnection, EntityTrait, Set};
use uuid::Uuid;

mod ent {
    use sea_orm::entity::prelude::*;
    use uuid::Uuid;

    #[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "pager_test")]
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
        Some(ent::Column::Id)
    }

    fn owner_col() -> Option<<Self as EntityTrait>::Column> {
        None
    }

    fn type_col() -> Option<<Self as EntityTrait>::Column> {
        None
    }
}

#[derive(Debug, Clone)]
struct TestDto {
    #[allow(dead_code)]
    id: i64,
    name: String,
    score: i64,
}

fn field_map() -> FieldMap<ent::Entity> {
    FieldMap::<ent::Entity>::new()
        .insert("id", ent::Column::Id, FieldKind::I64)
        .insert_with_extractor("id", ent::Column::Id, FieldKind::I64, |m: &ent::Model| {
            m.id.to_string()
        })
        .insert("tenant_id", ent::Column::TenantId, FieldKind::Uuid)
        .insert("name", ent::Column::Name, FieldKind::String)
        .insert_with_extractor(
            "name",
            ent::Column::Name,
            FieldKind::String,
            |m: &ent::Model| m.name.clone(),
        )
        .insert("score", ent::Column::Score, FieldKind::I64)
        .insert_with_extractor(
            "score",
            ent::Column::Score,
            FieldKind::I64,
            |m: &ent::Model| m.score.to_string(),
        )
}

async fn create_schema(conn: &DatabaseConnection) -> Result<()> {
    conn.execute_unprepared(
        "CREATE TABLE pager_test (
                id INTEGER PRIMARY KEY NOT NULL,
                tenant_id TEXT NOT NULL,
                name TEXT NOT NULL,
                score INTEGER NOT NULL
            )",
    )
    .await
    .map_err(|e| crate::DbError::Other(anyhow!(e.to_string())))?;
    Ok(())
}

async fn seed(conn: &DatabaseConnection, rows: &[(i64, Uuid, &str, i64)]) -> Result<()> {
    for (id, tenant_id, name, score) in rows {
        ent::Entity::insert(ent::ActiveModel {
            id: Set(*id),
            tenant_id: Set(*tenant_id),
            name: Set((*name).to_owned()),
            score: Set(*score),
        })
        .exec(conn)
        .await
        .map_err(|e| crate::DbError::Other(anyhow!(e.to_string())))?;
    }
    Ok(())
}

#[tokio::test]
async fn pager_applies_security_scope() {
    let (db, conn) = setup_sqlite_db().await.unwrap();
    create_schema(&conn).await.unwrap();

    let tenant1 = Uuid::new_v4();
    let tenant2 = Uuid::new_v4();

    seed(
        &conn,
        &[
            (1, tenant1, "alice", 10),
            (2, tenant1, "bob", 20),
            (3, tenant2, "charlie", 30),
            (4, tenant2, "dave", 40),
        ],
    )
    .await
    .unwrap();

    let scope = AccessScope::tenant(tenant1);
    let fmap = field_map();

    let query = ODataQuery {
        filter: None,
        order: ODataOrderBy(vec![]),
        limit: None,
        cursor: None,
        filter_hash: None,
        select: None,
    };

    let secure = db.sea_secure();
    let page = OPager::<ent::Entity, _>::new(&secure, &scope, &conn, &fmap)
        .tiebreaker("id", SortDir::Asc)
        .fetch(&query, |m| TestDto {
            id: m.id,
            name: m.name,
            score: m.score,
        })
        .await
        .unwrap();

    assert_eq!(page.items.len(), 2);
    assert_eq!(page.items[0].name, "alice");
    assert_eq!(page.items[1].name, "bob");
}

#[tokio::test]
async fn pager_respects_custom_tiebreaker() {
    let (db, conn) = setup_sqlite_db().await.unwrap();
    create_schema(&conn).await.unwrap();

    let tenant = Uuid::new_v4();

    seed(
        &conn,
        &[
            (1, tenant, "alice", 10),
            (2, tenant, "bob", 10),
            (3, tenant, "charlie", 10),
        ],
    )
    .await
    .unwrap();

    let scope = AccessScope::tenant(tenant);
    let fmap = field_map();

    let query = ODataQuery {
        filter: None,
        order: ODataOrderBy(vec![OrderKey {
            field: "score".to_owned(),
            dir: SortDir::Asc,
        }]),
        limit: None,
        cursor: None,
        filter_hash: None,
        select: None,
    };

    let secure = db.sea_secure();
    let page = OPager::<ent::Entity, _>::new(&secure, &scope, &conn, &fmap)
        .tiebreaker("name", SortDir::Asc)
        .fetch(&query, |m| TestDto {
            id: m.id,
            name: m.name,
            score: m.score,
        })
        .await
        .unwrap();

    assert_eq!(page.items.len(), 3);
    assert_eq!(page.items[0].name, "alice");
    assert_eq!(page.items[1].name, "bob");
    assert_eq!(page.items[2].name, "charlie");
}

#[tokio::test]
async fn pager_respects_custom_limits() {
    let (db, conn) = setup_sqlite_db().await.unwrap();
    create_schema(&conn).await.unwrap();

    let tenant = Uuid::new_v4();

    seed(
        &conn,
        &[
            (1, tenant, "alice", 10),
            (2, tenant, "bob", 20),
            (3, tenant, "charlie", 30),
            (4, tenant, "dave", 40),
            (5, tenant, "eve", 50),
        ],
    )
    .await
    .unwrap();

    let scope = AccessScope::tenant(tenant);
    let fmap = field_map();

    let query = ODataQuery {
        filter: None,
        order: ODataOrderBy(vec![]),
        limit: None,
        cursor: None,
        filter_hash: None,
        select: None,
    };

    let secure = db.sea_secure();
    let page = OPager::<ent::Entity, _>::new(&secure, &scope, &conn, &fmap)
        .tiebreaker("id", SortDir::Asc)
        .limits(2, 10)
        .fetch(&query, |m| TestDto {
            id: m.id,
            name: m.name,
            score: m.score,
        })
        .await
        .unwrap();

    assert_eq!(page.items.len(), 2);
    assert!(page.page_info.next_cursor.is_some());
}

#[tokio::test]
async fn pager_applies_odata_filter() {
    let (db, conn) = setup_sqlite_db().await.unwrap();
    create_schema(&conn).await.unwrap();

    let tenant = Uuid::new_v4();

    seed(
        &conn,
        &[
            (1, tenant, "alice", 10),
            (2, tenant, "bob", 20),
            (3, tenant, "charlie", 30),
        ],
    )
    .await
    .unwrap();

    let scope = AccessScope::tenant(tenant);
    let fmap = field_map();

    let filter_ast = modkit_odata::ast::Expr::Compare(
        Box::new(modkit_odata::ast::Expr::Identifier("score".to_owned())),
        modkit_odata::ast::CompareOperator::Gt,
        Box::new(modkit_odata::ast::Expr::Value(
            modkit_odata::ast::Value::Number(bigdecimal::BigDecimal::from(15)),
        )),
    );

    let query = ODataQuery {
        filter: Some(Box::new(filter_ast)),
        order: ODataOrderBy(vec![]),
        limit: None,
        cursor: None,
        filter_hash: None,
        select: None,
    };

    let secure = db.sea_secure();
    let page = OPager::<ent::Entity, _>::new(&secure, &scope, &conn, &fmap)
        .tiebreaker("id", SortDir::Asc)
        .fetch(&query, |m| TestDto {
            id: m.id,
            name: m.name,
            score: m.score,
        })
        .await
        .unwrap();

    assert_eq!(page.items.len(), 2);
    assert_eq!(page.items[0].name, "bob");
    assert_eq!(page.items[1].name, "charlie");
}

#[tokio::test]
async fn pager_applies_odata_order() {
    let (db, conn) = setup_sqlite_db().await.unwrap();
    create_schema(&conn).await.unwrap();

    let tenant = Uuid::new_v4();

    seed(
        &conn,
        &[
            (1, tenant, "alice", 30),
            (2, tenant, "bob", 10),
            (3, tenant, "charlie", 20),
        ],
    )
    .await
    .unwrap();

    let scope = AccessScope::tenant(tenant);
    let fmap = field_map();

    let query = ODataQuery {
        filter: None,
        order: ODataOrderBy(vec![OrderKey {
            field: "score".to_owned(),
            dir: SortDir::Asc,
        }]),
        limit: None,
        cursor: None,
        filter_hash: None,
        select: None,
    };

    let secure = db.sea_secure();
    let page = OPager::<ent::Entity, _>::new(&secure, &scope, &conn, &fmap)
        .fetch(&query, |m| TestDto {
            id: m.id,
            name: m.name,
            score: m.score,
        })
        .await
        .unwrap();

    assert_eq!(page.items.len(), 3);
    assert_eq!(page.items[0].score, 10);
    assert_eq!(page.items[1].score, 20);
    assert_eq!(page.items[2].score, 30);
}

#[tokio::test]
async fn pager_supports_cursor_pagination() {
    let (db, conn) = setup_sqlite_db().await.unwrap();
    create_schema(&conn).await.unwrap();

    let tenant = Uuid::new_v4();

    seed(
        &conn,
        &[
            (1, tenant, "alice", 10),
            (2, tenant, "bob", 20),
            (3, tenant, "charlie", 30),
            (4, tenant, "dave", 40),
        ],
    )
    .await
    .unwrap();

    let scope = AccessScope::tenant(tenant);
    let fmap = field_map();

    let query1 = ODataQuery {
        filter: None,
        order: ODataOrderBy(vec![OrderKey {
            field: "score".to_owned(),
            dir: SortDir::Asc,
        }]),
        limit: Some(2),
        cursor: None,
        filter_hash: None,
        select: None,
    };

    let secure = db.sea_secure();
    let page1 = OPager::<ent::Entity, _>::new(&secure, &scope, &conn, &fmap)
        .tiebreaker("id", SortDir::Asc)
        .fetch(&query1, |m| TestDto {
            id: m.id,
            name: m.name,
            score: m.score,
        })
        .await
        .unwrap();

    assert_eq!(page1.items.len(), 2);
    assert_eq!(page1.items[0].name, "alice");
    assert_eq!(page1.items[1].name, "bob");
    assert!(page1.page_info.next_cursor.is_some());

    let next_cursor =
        modkit_odata::CursorV1::decode(page1.page_info.next_cursor.as_deref().unwrap()).unwrap();

    let query2 = ODataQuery {
        filter: None,
        order: ODataOrderBy(vec![OrderKey {
            field: "score".to_owned(),
            dir: SortDir::Asc,
        }]),
        limit: Some(2),
        cursor: Some(next_cursor),
        filter_hash: None,
        select: None,
    };

    let page2 = OPager::<ent::Entity, _>::new(&secure, &scope, &conn, &fmap)
        .tiebreaker("id", SortDir::Asc)
        .fetch(&query2, |m| TestDto {
            id: m.id,
            name: m.name,
            score: m.score,
        })
        .await
        .unwrap();

    assert_eq!(page2.items.len(), 2);
    assert_eq!(page2.items[0].name, "charlie");
    assert_eq!(page2.items[1].name, "dave");
}

#[tokio::test]
async fn pager_empty_scope_returns_empty_results() {
    let (db, conn) = setup_sqlite_db().await.unwrap();
    create_schema(&conn).await.unwrap();

    let tenant1 = Uuid::new_v4();
    let tenant2 = Uuid::new_v4();

    seed(&conn, &[(1, tenant1, "alice", 10), (2, tenant1, "bob", 20)])
        .await
        .unwrap();

    let scope = AccessScope::tenant(tenant2);
    let fmap = field_map();

    let query = ODataQuery {
        filter: None,
        order: ODataOrderBy(vec![]),
        limit: None,
        cursor: None,
        filter_hash: None,
        select: None,
    };

    let secure = db.sea_secure();
    let page = OPager::<ent::Entity, _>::new(&secure, &scope, &conn, &fmap)
        .tiebreaker("id", SortDir::Asc)
        .fetch(&query, |m| TestDto {
            id: m.id,
            name: m.name,
            score: m.score,
        })
        .await
        .unwrap();

    assert_eq!(page.items.len(), 0);
    assert!(page.page_info.next_cursor.is_none());
    assert!(page.page_info.prev_cursor.is_none());
}

#[tokio::test]
async fn pager_combines_security_and_filter() {
    let (db, conn) = setup_sqlite_db().await.unwrap();
    create_schema(&conn).await.unwrap();

    let tenant1 = Uuid::new_v4();
    let tenant2 = Uuid::new_v4();

    seed(
        &conn,
        &[
            (1, tenant1, "alice", 10),
            (2, tenant1, "bob", 20),
            (3, tenant2, "charlie", 30),
            (4, tenant1, "dave", 40),
        ],
    )
    .await
    .unwrap();

    let scope = AccessScope::tenant(tenant1);
    let fmap = field_map();

    let filter_ast = modkit_odata::ast::Expr::Compare(
        Box::new(modkit_odata::ast::Expr::Identifier("score".to_owned())),
        modkit_odata::ast::CompareOperator::Ge,
        Box::new(modkit_odata::ast::Expr::Value(
            modkit_odata::ast::Value::Number(bigdecimal::BigDecimal::from(20)),
        )),
    );

    let query = ODataQuery {
        filter: Some(Box::new(filter_ast)),
        order: ODataOrderBy(vec![]),
        limit: None,
        cursor: None,
        filter_hash: None,
        select: None,
    };

    let secure = db.sea_secure();
    let page = OPager::<ent::Entity, _>::new(&secure, &scope, &conn, &fmap)
        .tiebreaker("id", SortDir::Asc)
        .fetch(&query, |m| TestDto {
            id: m.id,
            name: m.name,
            score: m.score,
        })
        .await
        .unwrap();

    assert_eq!(page.items.len(), 2);
    assert_eq!(page.items[0].name, "bob");
    assert_eq!(page.items[1].name, "dave");
}
