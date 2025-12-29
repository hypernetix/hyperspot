#![cfg(feature = "sea-orm")]

use super::core::{build_cursor_predicate, ensure_tiebreaker, expr_to_condition, ODataBuildError};
use super::{FieldKind, FieldMap};
use modkit_odata::{
    ast::{CompareOperator, Expr, Value},
    CursorV1, ODataOrderBy, OrderKey, SortDir,
};
use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "odata_tests")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: uuid::Uuid,
    pub name: String,
    pub score: i64,
    pub email: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

fn field_map() -> FieldMap<Entity> {
    FieldMap::<Entity>::new()
        .insert("id", Column::Id, FieldKind::Uuid)
        .insert("name", Column::Name, FieldKind::String)
        .insert("score", Column::Score, FieldKind::I64)
        .insert("email", Column::Email, FieldKind::String)
}

#[test]
fn ensure_tiebreaker_appends_when_missing() {
    // Arrange
    let order = ODataOrderBy(vec![OrderKey {
        field: "name".to_owned(),
        dir: SortDir::Asc,
    }]);

    // Act
    let order = ensure_tiebreaker(order, "id", SortDir::Desc);

    // Assert
    assert_eq!(order.0.len(), 2);
    assert_eq!(order.0[0].field, "name");
    assert_eq!(order.0[0].dir, SortDir::Asc);
    assert_eq!(order.0[1].field, "id");
    assert_eq!(order.0[1].dir, SortDir::Desc);
}

#[test]
fn ensure_tiebreaker_does_not_duplicate_when_present() {
    // Arrange
    let order = ODataOrderBy(vec![
        OrderKey {
            field: "name".to_owned(),
            dir: SortDir::Asc,
        },
        OrderKey {
            field: "id".to_owned(),
            dir: SortDir::Desc,
        },
    ]);

    // Act
    let order = ensure_tiebreaker(order, "id", SortDir::Asc);

    // Assert
    assert_eq!(order.0.len(), 2);
    assert_eq!(order.0[1].field, "id");
    assert_eq!(order.0[1].dir, SortDir::Desc);
}

#[test]
fn build_cursor_predicate_rejects_key_count_mismatch() {
    // Arrange
    let cursor = CursorV1 {
        k: vec!["a".to_owned()],
        o: SortDir::Asc,
        s: "+name".to_owned(),
        f: None,
        d: "fwd".to_owned(),
    };

    let order = ODataOrderBy(vec![
        OrderKey {
            field: "name".to_owned(),
            dir: SortDir::Asc,
        },
        OrderKey {
            field: "id".to_owned(),
            dir: SortDir::Desc,
        },
    ]);

    let fmap = field_map();

    // Act
    let err = build_cursor_predicate::<Entity>(&cursor, &order, &fmap).unwrap_err();

    // Assert
    assert!(matches!(
        err,
        ODataBuildError::Other("cursor keys count mismatch with order fields")
    ));
}

#[test]
fn build_cursor_predicate_rejects_unknown_order_field() {
    // Arrange
    let cursor = CursorV1 {
        k: vec!["a".to_owned()],
        o: SortDir::Asc,
        s: "+unknown".to_owned(),
        f: None,
        d: "fwd".to_owned(),
    };

    let order = ODataOrderBy(vec![OrderKey {
        field: "unknown".to_owned(),
        dir: SortDir::Asc,
    }]);

    let fmap = field_map();

    // Act
    let err = build_cursor_predicate::<Entity>(&cursor, &order, &fmap).unwrap_err();

    // Assert
    assert!(matches!(err, ODataBuildError::UnknownField(f) if f == "unknown"));
}

#[test]
fn expr_to_condition_rejects_bare_identifier() {
    // Arrange
    let expr = Expr::Identifier("name".to_owned());
    let fmap = field_map();

    // Act
    let err = expr_to_condition::<Entity>(&expr, &fmap).unwrap_err();

    // Assert
    assert!(matches!(err, ODataBuildError::BareIdentifier(f) if f == "name"));
}

#[test]
fn expr_to_condition_rejects_bare_literal() {
    // Arrange
    let expr = Expr::Value(Value::String("x".to_owned()));
    let fmap = field_map();

    // Act
    let err = expr_to_condition::<Entity>(&expr, &fmap).unwrap_err();

    // Assert
    assert!(matches!(err, ODataBuildError::BareLiteral));
}

#[test]
fn expr_to_condition_rejects_field_to_field_comparison() {
    // Arrange
    let expr = Expr::Compare(
        Box::new(Expr::Identifier("name".to_owned())),
        CompareOperator::Eq,
        Box::new(Expr::Identifier("email".to_owned())),
    );
    let fmap = field_map();

    // Act
    let err = expr_to_condition::<Entity>(&expr, &fmap).unwrap_err();

    // Assert
    assert!(matches!(
        err,
        ODataBuildError::Other("field-to-field comparison is not supported")
    ));
}

#[test]
fn expr_to_condition_allows_eq_null() {
    // Arrange
    let expr = Expr::Compare(
        Box::new(Expr::Identifier("email".to_owned())),
        CompareOperator::Eq,
        Box::new(Expr::Value(Value::Null)),
    );
    let fmap = field_map();

    // Act
    let cond = expr_to_condition::<Entity>(&expr, &fmap).unwrap();

    // Assert
    assert!(!cond.is_empty());
}

#[test]
fn expr_to_condition_rejects_non_equality_null_comparison() {
    // Arrange
    let expr = Expr::Compare(
        Box::new(Expr::Identifier("email".to_owned())),
        CompareOperator::Gt,
        Box::new(Expr::Value(Value::Null)),
    );
    let fmap = field_map();

    // Act
    let err = expr_to_condition::<Entity>(&expr, &fmap).unwrap_err();

    // Assert
    assert!(matches!(
        err,
        ODataBuildError::UnsupportedOp(CompareOperator::Gt)
    ));
}

#[test]
fn expr_to_condition_rejects_in_with_non_literal_list_items() {
    // Arrange
    let expr = Expr::In(
        Box::new(Expr::Identifier("score".to_owned())),
        vec![Expr::Identifier("score".to_owned())],
    );
    let fmap = field_map();

    // Act
    let err = expr_to_condition::<Entity>(&expr, &fmap).unwrap_err();

    // Assert
    assert!(matches!(err, ODataBuildError::NonLiteralInList));
}

#[test]
fn expr_to_condition_translates_empty_in_list_to_deny_all() {
    // Arrange
    let expr = Expr::In(Box::new(Expr::Identifier("score".to_owned())), vec![]);
    let fmap = field_map();

    // Act
    let cond = expr_to_condition::<Entity>(&expr, &fmap).unwrap();

    // Assert
    assert!(!cond.is_empty());
}

#[test]
fn expr_to_condition_rejects_type_mismatch_for_field_kind() {
    // Arrange
    let expr = Expr::Compare(
        Box::new(Expr::Identifier("score".to_owned())),
        CompareOperator::Eq,
        Box::new(Expr::Value(Value::String("not-a-number".to_owned()))),
    );
    let fmap = field_map();

    // Act
    let err = expr_to_condition::<Entity>(&expr, &fmap).unwrap_err();

    // Assert
    assert!(matches!(
        err,
        ODataBuildError::TypeMismatch {
            expected: FieldKind::I64,
            got: "string"
        }
    ));
}

#[test]
fn typed_cursor_value_uuid_round_trip() {
    // Arrange
    let id = uuid::Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap();
    let value = sea_orm::Value::Uuid(Some(Box::new(id)));

    // Act
    let encoded = super::sea_orm_filter::encode_cursor_value(&value, FieldKind::Uuid).unwrap();
    let decoded = super::sea_orm_filter::parse_cursor_value(FieldKind::Uuid, &encoded).unwrap();

    // Assert
    assert_eq!(encoded, "123e4567-e89b-12d3-a456-426614174000");
    assert!(matches!(decoded, sea_orm::Value::Uuid(Some(v)) if *v == id));
}

#[test]
fn typed_cursor_value_f64_round_trip() {
    // Arrange
    let value = sea_orm::Value::Double(Some(3.5));

    // Act
    let encoded = super::sea_orm_filter::encode_cursor_value(&value, FieldKind::F64).unwrap();
    let decoded = super::sea_orm_filter::parse_cursor_value(FieldKind::F64, &encoded).unwrap();

    // Assert
    assert_eq!(encoded, "3.5");
    assert!(matches!(decoded, sea_orm::Value::Double(Some(v)) if (v - 3.5).abs() < 1e-12));
}

#[cfg(feature = "sqlite")]
mod sqlite_pagination {
    use super::*;
    use crate::{ConnectOpts, DbHandle, Result};
    use anyhow::anyhow;
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
                Self::Id => FieldKind::I64,
                Self::Score => FieldKind::I64,
            }
        }
    }

    struct TMap;

    impl super::super::sea_orm_filter::FieldToColumn<TFld> for TMap {
        type Column = ent::Column;

        fn map_field(field: TFld) -> Self::Column {
            match field {
                TFld::Id => ent::Column::Id,
                TFld::Score => ent::Column::Score,
            }
        }
    }

    impl super::super::sea_orm_filter::ODataFieldMapping<TFld> for TMap {
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

    fn sqlite_mem_dsn(tag: &str) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        format!("sqlite:file:odata_{tag}_{now}?mode=memory&cache=shared")
    }

    async fn setup_db() -> Result<(DbHandle, DatabaseConnection)> {
        let dsn = sqlite_mem_dsn("paginate");
        let db = DbHandle::connect(&dsn, ConnectOpts::default()).await?;
        let conn = db.sea_secure().conn().clone();
        Ok((db, conn))
    }

    async fn create_schema(conn: &DatabaseConnection) -> Result<()> {
        conn.execute_unprepared(
            r#"CREATE TABLE odata_sqlite_tests (
                    id INTEGER PRIMARY KEY NOT NULL,
                    score INTEGER NOT NULL
                )"#,
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
        let (_db, conn) = setup_db().await.unwrap();
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

        let page = super::super::paginate_odata::<TFld, TMap, ent::Entity, i64, _, _>(
            ent::Entity::find(),
            &conn,
            &q,
            ("id", SortDir::Desc),
            super::super::LimitCfg {
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
        let (_db, conn) = setup_db().await.unwrap();
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

        let page1 = super::super::paginate_odata::<TFld, TMap, ent::Entity, i64, _, _>(
            ent::Entity::find(),
            &conn,
            &q1,
            ("id", SortDir::Desc),
            super::super::LimitCfg {
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

        let page2 = super::super::paginate_odata::<TFld, TMap, ent::Entity, i64, _, _>(
            ent::Entity::find(),
            &conn,
            &q2,
            ("id", SortDir::Desc),
            super::super::LimitCfg {
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
        let (_db, conn) = setup_db().await.unwrap();
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

        let page1 = super::super::paginate_odata::<TFld, TMap, ent::Entity, i64, _, _>(
            ent::Entity::find(),
            &conn,
            &q1,
            ("id", SortDir::Desc),
            super::super::LimitCfg {
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

        let page2 = super::super::paginate_odata::<TFld, TMap, ent::Entity, i64, _, _>(
            ent::Entity::find(),
            &conn,
            &q2,
            ("id", SortDir::Desc),
            super::super::LimitCfg {
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

        let page3 = super::super::paginate_odata::<TFld, TMap, ent::Entity, i64, _, _>(
            ent::Entity::find(),
            &conn,
            &q3,
            ("id", SortDir::Desc),
            super::super::LimitCfg {
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
}
