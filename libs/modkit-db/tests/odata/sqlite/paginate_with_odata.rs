use super::support::setup_sqlite_db;
use crate::odata::core::paginate_with_odata;
use crate::odata::{FieldKind, FieldMap, LimitCfg};
use crate::Result;
use anyhow::anyhow;
use modkit_odata::{ODataOrderBy, ODataQuery, OrderKey, SortDir};
use sea_orm::{ConnectionTrait, DatabaseConnection, EntityTrait, Set};

mod ent {
    use sea_orm::entity::prelude::*;

    #[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "paginate_test")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub name: String,
        pub score: i64,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

async fn create_schema(conn: &DatabaseConnection) -> Result<()> {
    conn.execute_unprepared(
        "CREATE TABLE paginate_test (
id INTEGER PRIMARY KEY NOT NULL,
name TEXT NOT NULL,
score INTEGER NOT NULL
)",
    )
    .await
    .map_err(|e| crate::DbError::Other(anyhow!(e.to_string())))?;
    Ok(())
}

async fn seed(conn: &DatabaseConnection, rows: &[(i64, &str, i64)]) -> Result<()> {
    for (id, name, score) in rows {
        ent::Entity::insert(ent::ActiveModel {
            id: Set(*id),
            name: Set((*name).to_owned()),
            score: Set(*score),
        })
        .exec(conn)
        .await
        .map_err(|e| crate::DbError::Other(anyhow!(e.to_string())))?;
    }
    Ok(())
}

fn field_map() -> FieldMap<ent::Entity> {
    fn id_extractor(m: &ent::Model) -> String {
        m.id.to_string()
    }
    fn name_extractor(m: &ent::Model) -> String {
        m.name.clone()
    }
    fn score_extractor(m: &ent::Model) -> String {
        m.score.to_string()
    }

    FieldMap::<ent::Entity>::new()
        .insert_with_extractor("id", ent::Column::Id, FieldKind::I64, id_extractor)
        .insert_with_extractor("name", ent::Column::Name, FieldKind::String, name_extractor)
        .insert_with_extractor("score", ent::Column::Score, FieldKind::I64, score_extractor)
}

#[tokio::test]
async fn paginate_with_odata_forward_first_page() {
    let (_db, conn) = setup_sqlite_db().await.unwrap();
    create_schema(&conn).await.unwrap();
    seed(
        &conn,
        &[
            (1, "alice", 10),
            (2, "bob", 20),
            (3, "charlie", 30),
            (4, "diana", 40),
        ],
    )
    .await
    .unwrap();

    let order = ODataOrderBy(vec![OrderKey {
        field: "score".to_owned(),
        dir: SortDir::Asc,
    }]);

    let q = ODataQuery {
        filter: None,
        order,
        limit: Some(2),
        cursor: None,
        filter_hash: None,
        select: None,
    };

    let page = paginate_with_odata(
        ent::Entity::find(),
        &conn,
        &q,
        &field_map(),
        ("id", SortDir::Desc),
        LimitCfg {
            default: 25,
            max: 1000,
        },
        |m| m,
    )
    .await
    .unwrap();

    assert_eq!(page.items.len(), 2);
    assert_eq!(page.items[0].name, "alice");
    assert_eq!(page.items[1].name, "bob");
    assert!(page.page_info.next_cursor.is_some());
    assert!(page.page_info.prev_cursor.is_none());
}

#[tokio::test]
async fn paginate_with_odata_forward_second_page_has_both_cursors() {
    let (_db, conn) = setup_sqlite_db().await.unwrap();
    create_schema(&conn).await.unwrap();
    seed(
        &conn,
        &[
            (1, "alice", 10),
            (2, "bob", 20),
            (3, "charlie", 30),
            (4, "diana", 40),
        ],
    )
    .await
    .unwrap();

    let order = ODataOrderBy(vec![OrderKey {
        field: "score".to_owned(),
        dir: SortDir::Asc,
    }]);

    let q1 = ODataQuery {
        filter: None,
        order: order.clone(),
        limit: Some(2),
        cursor: None,
        filter_hash: None,
        select: None,
    };

    let page1 = paginate_with_odata(
        ent::Entity::find(),
        &conn,
        &q1,
        &field_map(),
        ("id", SortDir::Desc),
        LimitCfg {
            default: 25,
            max: 1000,
        },
        |m| m,
    )
    .await
    .unwrap();

    let next_cursor = page1.page_info.next_cursor.unwrap();
    let cursor = modkit_odata::CursorV1::decode(&next_cursor).unwrap();

    let q2 = ODataQuery {
        filter: None,
        order,
        limit: Some(2),
        cursor: Some(cursor),
        filter_hash: None,
        select: None,
    };

    let page2 = paginate_with_odata(
        ent::Entity::find(),
        &conn,
        &q2,
        &field_map(),
        ("id", SortDir::Desc),
        LimitCfg {
            default: 25,
            max: 1000,
        },
        |m| m,
    )
    .await
    .unwrap();

    assert_eq!(page2.items.len(), 2);
    assert_eq!(page2.items[0].name, "charlie");
    assert_eq!(page2.items[1].name, "diana");
    assert!(page2.page_info.next_cursor.is_none());
    assert!(page2.page_info.prev_cursor.is_some());
}

#[tokio::test]
async fn paginate_with_odata_backward_from_end() {
    let (_db, conn) = setup_sqlite_db().await.unwrap();
    create_schema(&conn).await.unwrap();
    seed(
        &conn,
        &[
            (1, "alice", 10),
            (2, "bob", 20),
            (3, "charlie", 30),
            (4, "diana", 40),
        ],
    )
    .await
    .unwrap();

    let order = ODataOrderBy(vec![OrderKey {
        field: "score".to_owned(),
        dir: SortDir::Asc,
    }]);

    let q1 = ODataQuery {
        filter: None,
        order: order.clone(),
        limit: Some(2),
        cursor: None,
        filter_hash: None,
        select: None,
    };

    let page1 = paginate_with_odata(
        ent::Entity::find(),
        &conn,
        &q1,
        &field_map(),
        ("id", SortDir::Desc),
        LimitCfg {
            default: 25,
            max: 1000,
        },
        |m| m,
    )
    .await
    .unwrap();

    let next_cursor = page1.page_info.next_cursor.unwrap();
    let cursor = modkit_odata::CursorV1::decode(&next_cursor).unwrap();

    let q2 = ODataQuery {
        filter: None,
        order: order.clone(),
        limit: Some(2),
        cursor: Some(cursor),
        filter_hash: None,
        select: None,
    };

    let page2 = paginate_with_odata(
        ent::Entity::find(),
        &conn,
        &q2,
        &field_map(),
        ("id", SortDir::Desc),
        LimitCfg {
            default: 25,
            max: 1000,
        },
        |m| m,
    )
    .await
    .unwrap();

    let prev_cursor = page2.page_info.prev_cursor.unwrap();
    let cursor_bwd = modkit_odata::CursorV1::decode(&prev_cursor).unwrap();

    let q3 = ODataQuery {
        filter: None,
        order,
        limit: Some(2),
        cursor: Some(cursor_bwd),
        filter_hash: None,
        select: None,
    };

    let page3 = paginate_with_odata(
        ent::Entity::find(),
        &conn,
        &q3,
        &field_map(),
        ("id", SortDir::Desc),
        LimitCfg {
            default: 25,
            max: 1000,
        },
        |m| m,
    )
    .await
    .unwrap();

    assert_eq!(page3.items.len(), 2);
    assert_eq!(page3.items[0].name, "alice");
    assert_eq!(page3.items[1].name, "bob");
}

#[tokio::test]
async fn paginate_with_odata_respects_limit_clamp() {
    let (_db, conn) = setup_sqlite_db().await.unwrap();
    create_schema(&conn).await.unwrap();
    seed(
        &conn,
        &[(1, "alice", 10), (2, "bob", 20), (3, "charlie", 30)],
    )
    .await
    .unwrap();

    let order = ODataOrderBy(vec![OrderKey {
        field: "id".to_owned(),
        dir: SortDir::Asc,
    }]);

    let q = ODataQuery {
        filter: None,
        order,
        limit: Some(1000),
        cursor: None,
        filter_hash: None,
        select: None,
    };

    let page = paginate_with_odata(
        ent::Entity::find(),
        &conn,
        &q,
        &field_map(),
        ("id", SortDir::Desc),
        LimitCfg {
            default: 25,
            max: 2,
        },
        |m| m,
    )
    .await
    .unwrap();

    assert_eq!(page.items.len(), 2);
}

#[tokio::test]
async fn paginate_with_odata_applies_filter() {
    let (_db, conn) = setup_sqlite_db().await.unwrap();
    create_schema(&conn).await.unwrap();
    seed(
        &conn,
        &[
            (1, "alice", 10),
            (2, "bob", 20),
            (3, "charlie", 30),
            (4, "diana", 40),
        ],
    )
    .await
    .unwrap();

    let filter_ast = modkit_odata::ast::Expr::Compare(
        Box::new(modkit_odata::ast::Expr::Identifier("score".to_owned())),
        modkit_odata::ast::CompareOperator::Gt,
        Box::new(modkit_odata::ast::Expr::Value(
            modkit_odata::ast::Value::Number(bigdecimal::BigDecimal::from(15)),
        )),
    );

    let order = ODataOrderBy(vec![OrderKey {
        field: "score".to_owned(),
        dir: SortDir::Asc,
    }]);

    let q = ODataQuery {
        filter: Some(Box::new(filter_ast)),
        order,
        limit: Some(10),
        cursor: None,
        filter_hash: None,
        select: None,
    };

    let page = paginate_with_odata(
        ent::Entity::find(),
        &conn,
        &q,
        &field_map(),
        ("id", SortDir::Desc),
        LimitCfg {
            default: 25,
            max: 1000,
        },
        |m| m,
    )
    .await
    .unwrap();

    assert_eq!(page.items.len(), 3);
    assert_eq!(page.items[0].name, "bob");
    assert_eq!(page.items[1].name, "charlie");
    assert_eq!(page.items[2].name, "diana");
}
