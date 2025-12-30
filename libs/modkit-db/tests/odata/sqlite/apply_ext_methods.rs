use super::support::setup_sqlite_db;
use crate::odata::core::{CursorApplyExt, ODataExt, ODataOrderExt, ODataQueryExt};
use crate::odata::{FieldKind, FieldMap};
use crate::Result;
use anyhow::anyhow;
use modkit_odata::{CursorV1, ODataOrderBy, ODataQuery, OrderKey, SortDir};
use sea_orm::{ConnectionTrait, DatabaseConnection, EntityTrait, Set};

mod ent {
    use sea_orm::entity::prelude::*;

    #[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "apply_ext_test")]
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
        "CREATE TABLE apply_ext_test (
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
async fn apply_odata_filter_filters_results() {
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

    let results: Vec<i64> = ent::Entity::find()
        .apply_odata_filter(query, &field_map())
        .unwrap()
        .all(&conn)
        .await
        .unwrap()
        .into_iter()
        .map(|m| m.id)
        .collect();

    assert_eq!(results, vec![2, 3, 4]);
}

#[tokio::test]
async fn apply_odata_order_orders_results() {
    let (_db, conn) = setup_sqlite_db().await.unwrap();
    create_schema(&conn).await.unwrap();
    seed(
        &conn,
        &[(1, "charlie", 30), (2, "alice", 10), (3, "bob", 20)],
    )
    .await
    .unwrap();

    let order = ODataOrderBy(vec![OrderKey {
        field: "name".to_owned(),
        dir: SortDir::Asc,
    }]);

    let results: Vec<String> = ent::Entity::find()
        .apply_odata_order(&order, &field_map())
        .unwrap()
        .all(&conn)
        .await
        .unwrap()
        .into_iter()
        .map(|m| m.name)
        .collect();

    assert_eq!(results, vec!["alice", "bob", "charlie"]);
}

#[tokio::test]
async fn apply_odata_order_desc_reverses_order() {
    let (_db, conn) = setup_sqlite_db().await.unwrap();
    create_schema(&conn).await.unwrap();
    seed(
        &conn,
        &[(1, "alice", 10), (2, "bob", 20), (3, "charlie", 30)],
    )
    .await
    .unwrap();

    let order = ODataOrderBy(vec![OrderKey {
        field: "score".to_owned(),
        dir: SortDir::Desc,
    }]);

    let results: Vec<i64> = ent::Entity::find()
        .apply_odata_order(&order, &field_map())
        .unwrap()
        .all(&conn)
        .await
        .unwrap()
        .into_iter()
        .map(|m| m.score)
        .collect();

    assert_eq!(results, vec![30, 20, 10]);
}

#[tokio::test]
async fn apply_cursor_forward_filters_after_cursor() {
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

    let cursor = CursorV1 {
        k: vec!["20".to_owned()],
        o: SortDir::Asc,
        s: "+score".to_owned(),
        f: None,
        d: "fwd".to_owned(),
    };

    let results: Vec<String> = ent::Entity::find()
        .apply_cursor_forward(&cursor, &order, &field_map())
        .unwrap()
        .apply_odata_order(&order, &field_map())
        .unwrap()
        .all(&conn)
        .await
        .unwrap()
        .into_iter()
        .map(|m| m.name)
        .collect();

    assert_eq!(results, vec!["charlie", "diana"]);
}

#[tokio::test]
async fn apply_odata_query_combines_filter_cursor_order() {
    let (_db, conn) = setup_sqlite_db().await.unwrap();
    create_schema(&conn).await.unwrap();
    seed(
        &conn,
        &[
            (1, "alice", 10),
            (2, "bob", 20),
            (3, "charlie", 30),
            (4, "diana", 40),
            (5, "eve", 50),
        ],
    )
    .await
    .unwrap();

    let filter_ast = modkit_odata::ast::Expr::Compare(
        Box::new(modkit_odata::ast::Expr::Identifier("score".to_owned())),
        modkit_odata::ast::CompareOperator::Le,
        Box::new(modkit_odata::ast::Expr::Value(
            modkit_odata::ast::Value::Number(bigdecimal::BigDecimal::from(40)),
        )),
    );

    let cursor = CursorV1 {
        k: vec!["10".to_owned(), "1".to_owned()],
        o: SortDir::Asc,
        s: "+score,-id".to_owned(),
        f: None,
        d: "fwd".to_owned(),
    };

    let query = ODataQuery {
        filter: Some(Box::new(filter_ast)),
        order: ODataOrderBy(vec![OrderKey {
            field: "score".to_owned(),
            dir: SortDir::Asc,
        }]),
        limit: None,
        cursor: Some(cursor),
        filter_hash: None,
        select: None,
    };

    let results: Vec<String> = ent::Entity::find()
        .apply_odata_query(&query, &field_map(), ("id", SortDir::Desc))
        .unwrap()
        .all(&conn)
        .await
        .unwrap()
        .into_iter()
        .map(|m| m.name)
        .collect();

    assert_eq!(results, vec!["bob", "charlie", "diana"]);
}

#[tokio::test]
async fn apply_odata_query_with_tiebreaker() {
    let (_db, conn) = setup_sqlite_db().await.unwrap();
    create_schema(&conn).await.unwrap();
    seed(
        &conn,
        &[(1, "alice", 20), (2, "bob", 20), (3, "charlie", 20)],
    )
    .await
    .unwrap();

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

    let results: Vec<i64> = ent::Entity::find()
        .apply_odata_query(&query, &field_map(), ("id", SortDir::Desc))
        .unwrap()
        .all(&conn)
        .await
        .unwrap()
        .into_iter()
        .map(|m| m.id)
        .collect();

    assert_eq!(results, vec![3, 2, 1]);
}
