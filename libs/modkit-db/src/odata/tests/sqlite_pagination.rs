use super::support::setup_sqlite_db;
use crate::odata::FieldKind;
use crate::Result;
use anyhow::anyhow;
use modkit_odata::{CursorV1, ODataOrderBy, OrderKey, SortDir};
use sea_orm::{ConnectionTrait, DatabaseConnection, EntityTrait, Set};

mod ent {
    use sea_orm::entity::prelude::*;

    #[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "odata_sqlite_tests")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub score: i64,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum TFld {
    Id,
    Score,
}

impl crate::odata::FilterField for TFld {
    const FIELDS: &'static [Self] = &[Self::Id, Self::Score];

    fn name(&self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::Score => "score",
        }
    }

    fn kind(&self) -> FieldKind {
        match self {
            Self::Id | Self::Score => FieldKind::I64,
        }
    }
}

struct TMap;

impl crate::odata::sea_orm_filter::FieldToColumn<TFld> for TMap {
    type Column = ent::Column;

    fn map_field(field: TFld) -> Self::Column {
        match field {
            TFld::Id => ent::Column::Id,
            TFld::Score => ent::Column::Score,
        }
    }
}

impl crate::odata::sea_orm_filter::ODataFieldMapping<TFld> for TMap {
    type Entity = ent::Entity;

    fn extract_cursor_value(
        model: &<Self::Entity as EntityTrait>::Model,
        field: TFld,
    ) -> sea_orm::Value {
        match field {
            TFld::Id => sea_orm::Value::BigInt(Some(model.id)),
            TFld::Score => sea_orm::Value::BigInt(Some(model.score)),
        }
    }
}

async fn create_schema(conn: &DatabaseConnection) -> Result<()> {
    conn.execute_unprepared(
        "CREATE TABLE odata_sqlite_tests (
id INTEGER PRIMARY KEY NOT NULL,
score INTEGER NOT NULL
)",
    )
    .await
    .map_err(|e| crate::DbError::Other(anyhow!(e.to_string())))?;
    Ok(())
}

async fn seed(conn: &DatabaseConnection, rows: &[(i64, i64)]) -> Result<()> {
    for (id, score) in rows {
        ent::Entity::insert(ent::ActiveModel {
            id: Set(*id),
            score: Set(*score),
        })
        .exec(conn)
        .await
        .map_err(|e| crate::DbError::Other(anyhow!(e.to_string())))?;
    }
    Ok(())
}

#[tokio::test]
async fn paginate_odata_forward_first_page_yields_next_cursor_only() {
    let (_db, conn) = setup_sqlite_db().await.unwrap();
    create_schema(&conn).await.unwrap();
    seed(&conn, &[(1, 1), (2, 2), (3, 3), (4, 4), (5, 5)])
        .await
        .unwrap();

    let order = ODataOrderBy(vec![OrderKey {
        field: "score".to_owned(),
        dir: SortDir::Asc,
    }]);

    let q = modkit_odata::ODataQuery {
        filter: None,
        order,
        limit: Some(2),
        cursor: None,
        filter_hash: None,
        select: None,
    };

    let page = crate::odata::paginate_odata::<TFld, TMap, ent::Entity, i64, _, _>(
        ent::Entity::find(),
        &conn,
        &q,
        ("id", SortDir::Desc),
        crate::odata::LimitCfg {
            default: 25,
            max: 1000,
        },
        |m| m.id,
    )
    .await
    .unwrap();

    assert_eq!(page.items, vec![1, 2]);
    assert!(page.page_info.next_cursor.is_some());
    assert!(page.page_info.prev_cursor.is_none());

    let next = CursorV1::decode(page.page_info.next_cursor.as_deref().unwrap()).unwrap();
    assert_eq!(next.d, "fwd");
}

#[tokio::test]
async fn paginate_odata_forward_second_page_has_prev_cursor() {
    let (_db, conn) = setup_sqlite_db().await.unwrap();
    create_schema(&conn).await.unwrap();
    seed(&conn, &[(1, 1), (2, 2), (3, 3), (4, 4), (5, 5)])
        .await
        .unwrap();

    let order = ODataOrderBy(vec![OrderKey {
        field: "score".to_owned(),
        dir: SortDir::Asc,
    }]);

    let q1 = modkit_odata::ODataQuery {
        filter: None,
        order: order.clone(),
        limit: Some(2),
        cursor: None,
        filter_hash: None,
        select: None,
    };

    let page1 = crate::odata::paginate_odata::<TFld, TMap, ent::Entity, i64, _, _>(
        ent::Entity::find(),
        &conn,
        &q1,
        ("id", SortDir::Desc),
        crate::odata::LimitCfg {
            default: 25,
            max: 1000,
        },
        |m| m.id,
    )
    .await
    .unwrap();

    let next1 = CursorV1::decode(page1.page_info.next_cursor.as_deref().unwrap()).unwrap();
    assert_eq!(next1.d, "fwd");

    let q2 = modkit_odata::ODataQuery {
        filter: None,
        order,
        limit: Some(2),
        cursor: Some(next1),
        filter_hash: None,
        select: None,
    };

    let page2 = crate::odata::paginate_odata::<TFld, TMap, ent::Entity, i64, _, _>(
        ent::Entity::find(),
        &conn,
        &q2,
        ("id", SortDir::Desc),
        crate::odata::LimitCfg {
            default: 25,
            max: 1000,
        },
        |m| m.id,
    )
    .await
    .unwrap();

    assert_eq!(page2.items, vec![3, 4]);
    assert!(page2.page_info.next_cursor.is_some());
    assert!(page2.page_info.prev_cursor.is_some());

    let prev2 = CursorV1::decode(page2.page_info.prev_cursor.as_deref().unwrap()).unwrap();
    assert_eq!(prev2.d, "bwd");
}

#[tokio::test]
async fn paginate_odata_backward_from_second_page_returns_first_page() {
    let (_db, conn) = setup_sqlite_db().await.unwrap();
    create_schema(&conn).await.unwrap();
    seed(&conn, &[(1, 1), (2, 2), (3, 3), (4, 4), (5, 5)])
        .await
        .unwrap();

    let order = ODataOrderBy(vec![OrderKey {
        field: "score".to_owned(),
        dir: SortDir::Asc,
    }]);

    let q1 = modkit_odata::ODataQuery {
        filter: None,
        order: order.clone(),
        limit: Some(2),
        cursor: None,
        filter_hash: None,
        select: None,
    };

    let page1 = crate::odata::paginate_odata::<TFld, TMap, ent::Entity, i64, _, _>(
        ent::Entity::find(),
        &conn,
        &q1,
        ("id", SortDir::Desc),
        crate::odata::LimitCfg {
            default: 25,
            max: 1000,
        },
        |m| m.id,
    )
    .await
    .unwrap();

    let next1 = CursorV1::decode(page1.page_info.next_cursor.as_deref().unwrap()).unwrap();

    let q2 = modkit_odata::ODataQuery {
        filter: None,
        order,
        limit: Some(2),
        cursor: Some(next1),
        filter_hash: None,
        select: None,
    };

    let page2 = crate::odata::paginate_odata::<TFld, TMap, ent::Entity, i64, _, _>(
        ent::Entity::find(),
        &conn,
        &q2,
        ("id", SortDir::Desc),
        crate::odata::LimitCfg {
            default: 25,
            max: 1000,
        },
        |m| m.id,
    )
    .await
    .unwrap();

    let prev2 = CursorV1::decode(page2.page_info.prev_cursor.as_deref().unwrap()).unwrap();
    assert_eq!(prev2.d, "bwd");

    let q3 = modkit_odata::ODataQuery {
        filter: None,
        order: ODataOrderBy(vec![]),
        limit: Some(2),
        cursor: Some(prev2),
        filter_hash: None,
        select: None,
    };

    let page3 = crate::odata::paginate_odata::<TFld, TMap, ent::Entity, i64, _, _>(
        ent::Entity::find(),
        &conn,
        &q3,
        ("id", SortDir::Desc),
        crate::odata::LimitCfg {
            default: 25,
            max: 1000,
        },
        |m| m.id,
    )
    .await
    .unwrap();

    assert_eq!(page3.items, vec![1, 2]);
    assert!(page3.page_info.next_cursor.is_some());
    assert!(page3.page_info.prev_cursor.is_none());

    let next3 = CursorV1::decode(page3.page_info.next_cursor.as_deref().unwrap()).unwrap();
    assert_eq!(next3.d, "fwd");
}
