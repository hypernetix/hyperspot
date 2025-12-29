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

#[cfg(feature = "sqlite")]
mod expr_to_condition_sqlite {
    use super::*;
    use crate::{ConnectOpts, DbHandle, Result};
    use anyhow::anyhow;
    use bigdecimal::BigDecimal;
    use chrono::{NaiveDate, NaiveTime, Utc};
    use modkit_odata::ast as odata_ast;
    use rust_decimal::Decimal;
    use sea_orm::{ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
    use std::str::FromStr;
    use uuid::Uuid;

    mod ent {
        use sea_orm::entity::prelude::*;

        #[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
        #[sea_orm(table_name = "expr_test")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i64,
            pub name: String,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        impl ActiveModelBehavior for ActiveModel {}
    }

    fn sqlite_mem_dsn(tag: &str) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        format!("sqlite:file:expr_{tag}_{now}?mode=memory&cache=shared")
    }

    async fn setup_db() -> Result<(DbHandle, DatabaseConnection)> {
        let dsn = sqlite_mem_dsn("test");
        let db = DbHandle::connect(&dsn, ConnectOpts::default()).await?;
        let conn = db.sea_secure().conn().clone();
        Ok((db, conn))
    }

    async fn create_schema(conn: &DatabaseConnection) -> Result<()> {
        conn.execute_unprepared(
            r#"CREATE TABLE expr_test (
                    id INTEGER PRIMARY KEY NOT NULL,
                    name TEXT NOT NULL
                )"#,
        )
        .await
        .map_err(|e| crate::DbError::Other(anyhow!(e.to_string())))?;
        Ok(())
    }

    async fn seed(conn: &DatabaseConnection, rows: &[(i64, &str)]) -> Result<()> {
        for (id, name) in rows {
            ent::Entity::insert(ent::ActiveModel {
                id: Set(*id),
                name: Set((*name).to_owned()),
            })
            .exec(conn)
            .await
            .map_err(|e| crate::DbError::Other(anyhow!(e.to_string())))?;
        }
        Ok(())
    }

    fn field_map() -> FieldMap<ent::Entity> {
        FieldMap::<ent::Entity>::new()
            .insert("id", ent::Column::Id, FieldKind::I64)
            .insert("name", ent::Column::Name, FieldKind::String)
    }

    mod ent_decimal {
        use sea_orm::entity::prelude::*;

        #[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
        #[sea_orm(table_name = "expr_decimal_test")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i64,
            pub amount: rust_decimal::Decimal,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        impl ActiveModelBehavior for ActiveModel {}
    }

    async fn create_schema_decimal(conn: &DatabaseConnection) -> Result<()> {
        conn.execute_unprepared(
            r#"CREATE TABLE expr_decimal_test (
                    id INTEGER PRIMARY KEY NOT NULL,
                    amount NUMERIC NOT NULL
                )"#,
        )
        .await
        .map_err(|e| crate::DbError::Other(anyhow!(e.to_string())))?;
        Ok(())
    }

    async fn seed_decimal(conn: &DatabaseConnection, rows: &[(i64, Decimal)]) -> Result<()> {
        for (id, amount) in rows {
            ent_decimal::Entity::insert(ent_decimal::ActiveModel {
                id: Set(*id),
                amount: Set(*amount),
            })
            .exec(conn)
            .await
            .map_err(|e| crate::DbError::Other(anyhow!(e.to_string())))?;
        }
        Ok(())
    }

    fn field_map_decimal() -> FieldMap<ent_decimal::Entity> {
        FieldMap::<ent_decimal::Entity>::new()
            .insert("id", ent_decimal::Column::Id, FieldKind::I64)
            .insert("amount", ent_decimal::Column::Amount, FieldKind::Decimal)
    }

    mod ent_coerce {
        use sea_orm::entity::prelude::*;

        #[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
        #[sea_orm(table_name = "expr_coerce_test")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i64,
            pub s: String,
            pub i: i64,
            pub f: f64,
            pub b: bool,
            pub u: uuid::Uuid,
            pub dt: chrono::DateTime<chrono::Utc>,
            pub d: chrono::NaiveDate,
            pub t: chrono::NaiveTime,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        impl ActiveModelBehavior for ActiveModel {}
    }

    async fn create_schema_coerce(conn: &DatabaseConnection) -> Result<()> {
        conn.execute_unprepared(
            r#"CREATE TABLE expr_coerce_test (
                    id INTEGER PRIMARY KEY NOT NULL,
                    s TEXT NOT NULL,
                    i INTEGER NOT NULL,
                    f REAL NOT NULL,
                    b INTEGER NOT NULL,
                    u BLOB NOT NULL,
                    dt TEXT NOT NULL,
                    d TEXT NOT NULL,
                    t TEXT NOT NULL
                )"#,
        )
        .await
        .map_err(|e| crate::DbError::Other(anyhow!(e.to_string())))?;
        Ok(())
    }

    async fn seed_coerce(conn: &DatabaseConnection, row: ent_coerce::Model) -> Result<()> {
        ent_coerce::Entity::insert(ent_coerce::ActiveModel {
            id: Set(row.id),
            s: Set(row.s),
            i: Set(row.i),
            f: Set(row.f),
            b: Set(row.b),
            u: Set(row.u),
            dt: Set(row.dt),
            d: Set(row.d),
            t: Set(row.t),
        })
        .exec(conn)
        .await
        .map_err(|e| crate::DbError::Other(anyhow!(e.to_string())))?;
        Ok(())
    }

    fn field_map_coerce() -> FieldMap<ent_coerce::Entity> {
        FieldMap::<ent_coerce::Entity>::new()
            .insert("s", ent_coerce::Column::S, FieldKind::String)
            .insert("i", ent_coerce::Column::I, FieldKind::I64)
            .insert("f", ent_coerce::Column::F, FieldKind::F64)
            .insert("b", ent_coerce::Column::B, FieldKind::Bool)
            .insert("u", ent_coerce::Column::U, FieldKind::Uuid)
            .insert("dt", ent_coerce::Column::Dt, FieldKind::DateTimeUtc)
            .insert("d", ent_coerce::Column::D, FieldKind::Date)
            .insert("t", ent_coerce::Column::T, FieldKind::Time)
    }

    struct CoerceFixture {
        u: Uuid,
        dt: chrono::DateTime<Utc>,
        d: NaiveDate,
        t: NaiveTime,
    }

    async fn setup_coerce_fixture() -> Result<(DbHandle, DatabaseConnection, CoerceFixture)> {
        let (db, conn) = setup_db().await?;
        create_schema_coerce(&conn).await?;

        let u = Uuid::new_v4();
        let dt = chrono::DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
            .map_err(|_| crate::DbError::Other(anyhow!("invalid datetime")))?
            .with_timezone(&Utc);
        let d = NaiveDate::from_ymd_opt(2024, 1, 2)
            .ok_or_else(|| crate::DbError::Other(anyhow!("invalid date")))?;
        let t = NaiveTime::from_hms_opt(3, 4, 5)
            .ok_or_else(|| crate::DbError::Other(anyhow!("invalid time")))?;

        seed_coerce(
            &conn,
            ent_coerce::Model {
                id: 1,
                s: "hello".to_owned(),
                i: 42,
                f: 1.25,
                b: true,
                u,
                dt,
                d,
                t,
            },
        )
        .await?;

        Ok((db, conn, CoerceFixture { u, dt, d, t }))
    }

    // ===== Logical operators =====

    #[tokio::test]
    async fn expr_and_filters_both_conditions() {
        // Arrange
        let (_db, conn) = setup_db().await.unwrap();
        create_schema(&conn).await.unwrap();
        seed(
            &conn,
            &[(1, "alice"), (2, "bob"), (10, "alice"), (20, "bob")],
        )
        .await
        .unwrap();

        let expr = odata_ast::Expr::And(
            Box::new(odata_ast::Expr::Compare(
                Box::new(odata_ast::Expr::Identifier("name".to_owned())),
                odata_ast::CompareOperator::Eq,
                Box::new(odata_ast::Expr::Value(odata_ast::Value::String(
                    "alice".to_owned(),
                ))),
            )),
            Box::new(odata_ast::Expr::Compare(
                Box::new(odata_ast::Expr::Identifier("id".to_owned())),
                odata_ast::CompareOperator::Gt,
                Box::new(odata_ast::Expr::Value(odata_ast::Value::Number(
                    BigDecimal::from(5),
                ))),
            )),
        );

        let fmap = field_map();

        // Act
        let cond = expr_to_condition::<ent::Entity>(&expr, &fmap).unwrap();
        let results = ent::Entity::find().filter(cond).all(&conn).await.unwrap();

        // Assert
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, 10);
        assert_eq!(results[0].name, "alice");
    }

    #[tokio::test]
    async fn expr_or_matches_either_condition() {
        // Arrange
        let (_db, conn) = setup_db().await.unwrap();
        create_schema(&conn).await.unwrap();
        seed(&conn, &[(1, "alice"), (2, "bob"), (3, "charlie")])
            .await
            .unwrap();

        let expr = odata_ast::Expr::Or(
            Box::new(odata_ast::Expr::Compare(
                Box::new(odata_ast::Expr::Identifier("name".to_owned())),
                odata_ast::CompareOperator::Eq,
                Box::new(odata_ast::Expr::Value(odata_ast::Value::String(
                    "alice".to_owned(),
                ))),
            )),
            Box::new(odata_ast::Expr::Compare(
                Box::new(odata_ast::Expr::Identifier("name".to_owned())),
                odata_ast::CompareOperator::Eq,
                Box::new(odata_ast::Expr::Value(odata_ast::Value::String(
                    "bob".to_owned(),
                ))),
            )),
        );

        let fmap = field_map();

        // Act
        let cond = expr_to_condition::<ent::Entity>(&expr, &fmap).unwrap();
        let mut results = ent::Entity::find().filter(cond).all(&conn).await.unwrap();
        results.sort_by_key(|r| r.id);

        // Assert
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].name, "alice");
        assert_eq!(results[1].name, "bob");
    }

    #[tokio::test]
    async fn expr_not_inverts_condition() {
        // Arrange
        let (_db, conn) = setup_db().await.unwrap();
        create_schema(&conn).await.unwrap();
        seed(&conn, &[(1, "alice"), (2, "bob"), (3, "charlie")])
            .await
            .unwrap();

        let expr = odata_ast::Expr::Not(Box::new(odata_ast::Expr::Compare(
            Box::new(odata_ast::Expr::Identifier("name".to_owned())),
            odata_ast::CompareOperator::Eq,
            Box::new(odata_ast::Expr::Value(odata_ast::Value::String(
                "bob".to_owned(),
            ))),
        )));

        let fmap = field_map();

        // Act
        let cond = expr_to_condition::<ent::Entity>(&expr, &fmap).unwrap();
        let mut results = ent::Entity::find().filter(cond).all(&conn).await.unwrap();
        results.sort_by_key(|r| r.id);

        // Assert
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].name, "alice");
        assert_eq!(results[1].name, "charlie");
    }

    // ===== Compare operators =====

    #[tokio::test]
    async fn expr_compare_eq_matches_exact() {
        // Arrange
        let (_db, conn) = setup_db().await.unwrap();
        create_schema(&conn).await.unwrap();
        seed(&conn, &[(1, "alice"), (2, "bob")]).await.unwrap();

        let expr = odata_ast::Expr::Compare(
            Box::new(odata_ast::Expr::Identifier("name".to_owned())),
            odata_ast::CompareOperator::Eq,
            Box::new(odata_ast::Expr::Value(odata_ast::Value::String(
                "alice".to_owned(),
            ))),
        );

        let fmap = field_map();

        // Act
        let cond = expr_to_condition::<ent::Entity>(&expr, &fmap).unwrap();
        let results = ent::Entity::find().filter(cond).all(&conn).await.unwrap();

        // Assert
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "alice");
    }

    #[tokio::test]
    async fn expr_compare_ne_excludes_match() {
        // Arrange
        let (_db, conn) = setup_db().await.unwrap();
        create_schema(&conn).await.unwrap();
        seed(&conn, &[(1, "alice"), (2, "bob")]).await.unwrap();

        let expr = odata_ast::Expr::Compare(
            Box::new(odata_ast::Expr::Identifier("name".to_owned())),
            odata_ast::CompareOperator::Ne,
            Box::new(odata_ast::Expr::Value(odata_ast::Value::String(
                "alice".to_owned(),
            ))),
        );

        let fmap = field_map();

        // Act
        let cond = expr_to_condition::<ent::Entity>(&expr, &fmap).unwrap();
        let results = ent::Entity::find().filter(cond).all(&conn).await.unwrap();

        // Assert
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "bob");
    }

    #[tokio::test]
    async fn expr_compare_gt_filters_greater() {
        // Arrange
        let (_db, conn) = setup_db().await.unwrap();
        create_schema(&conn).await.unwrap();
        seed(&conn, &[(5, "a"), (10, "b"), (15, "c")])
            .await
            .unwrap();

        let expr = odata_ast::Expr::Compare(
            Box::new(odata_ast::Expr::Identifier("id".to_owned())),
            odata_ast::CompareOperator::Gt,
            Box::new(odata_ast::Expr::Value(odata_ast::Value::Number(
                BigDecimal::from(10),
            ))),
        );

        let fmap = field_map();

        // Act
        let cond = expr_to_condition::<ent::Entity>(&expr, &fmap).unwrap();
        let results = ent::Entity::find().filter(cond).all(&conn).await.unwrap();

        // Assert
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, 15);
    }

    #[tokio::test]
    async fn expr_compare_ge_includes_equal() {
        // Arrange
        let (_db, conn) = setup_db().await.unwrap();
        create_schema(&conn).await.unwrap();
        seed(&conn, &[(5, "a"), (10, "b"), (15, "c")])
            .await
            .unwrap();

        let expr = odata_ast::Expr::Compare(
            Box::new(odata_ast::Expr::Identifier("id".to_owned())),
            odata_ast::CompareOperator::Ge,
            Box::new(odata_ast::Expr::Value(odata_ast::Value::Number(
                BigDecimal::from(10),
            ))),
        );

        let fmap = field_map();

        // Act
        let cond = expr_to_condition::<ent::Entity>(&expr, &fmap).unwrap();
        let mut results = ent::Entity::find().filter(cond).all(&conn).await.unwrap();
        results.sort_by_key(|r| r.id);

        // Assert
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, 10);
        assert_eq!(results[1].id, 15);
    }

    #[tokio::test]
    async fn expr_compare_lt_filters_less() {
        // Arrange
        let (_db, conn) = setup_db().await.unwrap();
        create_schema(&conn).await.unwrap();
        seed(&conn, &[(5, "a"), (10, "b"), (15, "c")])
            .await
            .unwrap();

        let expr = odata_ast::Expr::Compare(
            Box::new(odata_ast::Expr::Identifier("id".to_owned())),
            odata_ast::CompareOperator::Lt,
            Box::new(odata_ast::Expr::Value(odata_ast::Value::Number(
                BigDecimal::from(10),
            ))),
        );

        let fmap = field_map();

        // Act
        let cond = expr_to_condition::<ent::Entity>(&expr, &fmap).unwrap();
        let results = ent::Entity::find().filter(cond).all(&conn).await.unwrap();

        // Assert
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, 5);
    }

    #[tokio::test]
    async fn expr_compare_le_includes_equal() {
        // Arrange
        let (_db, conn) = setup_db().await.unwrap();
        create_schema(&conn).await.unwrap();
        seed(&conn, &[(5, "a"), (10, "b"), (15, "c")])
            .await
            .unwrap();

        let expr = odata_ast::Expr::Compare(
            Box::new(odata_ast::Expr::Identifier("id".to_owned())),
            odata_ast::CompareOperator::Le,
            Box::new(odata_ast::Expr::Value(odata_ast::Value::Number(
                BigDecimal::from(10),
            ))),
        );

        let fmap = field_map();

        // Act
        let cond = expr_to_condition::<ent::Entity>(&expr, &fmap).unwrap();
        let mut results = ent::Entity::find().filter(cond).all(&conn).await.unwrap();
        results.sort_by_key(|r| r.id);

        // Assert
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, 5);
        assert_eq!(results[1].id, 10);
    }

    #[tokio::test]
    async fn expr_compare_decimal_eq_uses_bigdecimal_to_decimal() {
        // Arrange
        let (_db, conn) = setup_db().await.unwrap();
        create_schema_decimal(&conn).await.unwrap();
        seed_decimal(
            &conn,
            &[
                (1, Decimal::from_str_exact("10.5").unwrap()),
                (2, Decimal::from_str_exact("20").unwrap()),
            ],
        )
        .await
        .unwrap();

        let expr = odata_ast::Expr::Compare(
            Box::new(odata_ast::Expr::Identifier("amount".to_owned())),
            odata_ast::CompareOperator::Eq,
            Box::new(odata_ast::Expr::Value(odata_ast::Value::Number(
                BigDecimal::from_str("10.50").unwrap(),
            ))),
        );

        let fmap = field_map_decimal();

        // Act
        let cond = expr_to_condition::<ent_decimal::Entity>(&expr, &fmap).unwrap();
        let results = ent_decimal::Entity::find()
            .filter(cond)
            .all(&conn)
            .await
            .unwrap();

        // Assert
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, 1);
        assert_eq!(results[0].amount, Decimal::from_str_exact("10.5").unwrap());
    }

    #[tokio::test]
    async fn expr_to_condition_coerce_string() {
        // Arrange
        let (_db, conn, _fx) = setup_coerce_fixture().await.unwrap();
        let fmap = field_map_coerce();

        let expr = odata_ast::Expr::Compare(
            Box::new(odata_ast::Expr::Identifier("s".to_owned())),
            odata_ast::CompareOperator::Eq,
            Box::new(odata_ast::Expr::Value(odata_ast::Value::String(
                "hello".to_owned(),
            ))),
        );

        // Act
        let cond = expr_to_condition::<ent_coerce::Entity>(&expr, &fmap).unwrap();
        let ids: Vec<i64> = ent_coerce::Entity::find()
            .filter(cond)
            .all(&conn)
            .await
            .unwrap()
            .into_iter()
            .map(|m| m.id)
            .collect();

        // Assert
        assert_eq!(ids, vec![1]);
    }

    #[tokio::test]
    async fn expr_to_condition_coerce_i64() {
        // Arrange
        let (_db, conn, _fx) = setup_coerce_fixture().await.unwrap();
        let fmap = field_map_coerce();
        let expr = odata_ast::Expr::Compare(
            Box::new(odata_ast::Expr::Identifier("i".to_owned())),
            odata_ast::CompareOperator::Eq,
            Box::new(odata_ast::Expr::Value(odata_ast::Value::Number(
                BigDecimal::from(42),
            ))),
        );

        // Act
        let cond = expr_to_condition::<ent_coerce::Entity>(&expr, &fmap).unwrap();
        let ids: Vec<i64> = ent_coerce::Entity::find()
            .filter(cond)
            .all(&conn)
            .await
            .unwrap()
            .into_iter()
            .map(|m| m.id)
            .collect();

        // Assert
        assert_eq!(ids, vec![1]);
    }

    #[tokio::test]
    async fn expr_to_condition_coerce_f64() {
        // Arrange
        let (_db, conn, _fx) = setup_coerce_fixture().await.unwrap();
        let fmap = field_map_coerce();
        let expr = odata_ast::Expr::Compare(
            Box::new(odata_ast::Expr::Identifier("f".to_owned())),
            odata_ast::CompareOperator::Eq,
            Box::new(odata_ast::Expr::Value(odata_ast::Value::Number(
                BigDecimal::from_str("1.25").unwrap(),
            ))),
        );

        // Act
        let cond = expr_to_condition::<ent_coerce::Entity>(&expr, &fmap).unwrap();
        let ids: Vec<i64> = ent_coerce::Entity::find()
            .filter(cond)
            .all(&conn)
            .await
            .unwrap()
            .into_iter()
            .map(|m| m.id)
            .collect();

        // Assert
        assert_eq!(ids, vec![1]);
    }

    #[tokio::test]
    async fn expr_to_condition_coerce_bool() {
        // Arrange
        let (_db, conn, _fx) = setup_coerce_fixture().await.unwrap();
        let fmap = field_map_coerce();
        let expr = odata_ast::Expr::Compare(
            Box::new(odata_ast::Expr::Identifier("b".to_owned())),
            odata_ast::CompareOperator::Eq,
            Box::new(odata_ast::Expr::Value(odata_ast::Value::Bool(true))),
        );

        // Act
        let cond = expr_to_condition::<ent_coerce::Entity>(&expr, &fmap).unwrap();
        let ids: Vec<i64> = ent_coerce::Entity::find()
            .filter(cond)
            .all(&conn)
            .await
            .unwrap()
            .into_iter()
            .map(|m| m.id)
            .collect();

        // Assert
        assert_eq!(ids, vec![1]);
    }

    #[tokio::test]
    async fn expr_to_condition_coerce_uuid() {
        // Arrange
        let (_db, conn, fx) = setup_coerce_fixture().await.unwrap();
        let fmap = field_map_coerce();
        let expr = odata_ast::Expr::Compare(
            Box::new(odata_ast::Expr::Identifier("u".to_owned())),
            odata_ast::CompareOperator::Eq,
            Box::new(odata_ast::Expr::Value(odata_ast::Value::Uuid(fx.u))),
        );

        // Act
        let cond = expr_to_condition::<ent_coerce::Entity>(&expr, &fmap).unwrap();
        let ids: Vec<i64> = ent_coerce::Entity::find()
            .filter(cond)
            .all(&conn)
            .await
            .unwrap()
            .into_iter()
            .map(|m| m.id)
            .collect();

        // Assert
        assert_eq!(ids, vec![1]);
    }

    #[tokio::test]
    async fn expr_to_condition_coerce_datetime_utc() {
        // Arrange
        let (_db, conn, fx) = setup_coerce_fixture().await.unwrap();
        let fmap = field_map_coerce();
        let expr = odata_ast::Expr::Compare(
            Box::new(odata_ast::Expr::Identifier("dt".to_owned())),
            odata_ast::CompareOperator::Eq,
            Box::new(odata_ast::Expr::Value(odata_ast::Value::DateTime(fx.dt))),
        );

        // Act
        let cond = expr_to_condition::<ent_coerce::Entity>(&expr, &fmap).unwrap();
        let ids: Vec<i64> = ent_coerce::Entity::find()
            .filter(cond)
            .all(&conn)
            .await
            .unwrap()
            .into_iter()
            .map(|m| m.id)
            .collect();

        // Assert
        assert_eq!(ids, vec![1]);
    }

    #[tokio::test]
    async fn expr_to_condition_coerce_date() {
        // Arrange
        let (_db, conn, fx) = setup_coerce_fixture().await.unwrap();
        let fmap = field_map_coerce();
        let expr = odata_ast::Expr::Compare(
            Box::new(odata_ast::Expr::Identifier("d".to_owned())),
            odata_ast::CompareOperator::Eq,
            Box::new(odata_ast::Expr::Value(odata_ast::Value::Date(fx.d))),
        );

        // Act
        let cond = expr_to_condition::<ent_coerce::Entity>(&expr, &fmap).unwrap();
        let ids: Vec<i64> = ent_coerce::Entity::find()
            .filter(cond)
            .all(&conn)
            .await
            .unwrap()
            .into_iter()
            .map(|m| m.id)
            .collect();

        // Assert
        assert_eq!(ids, vec![1]);
    }

    #[tokio::test]
    async fn expr_to_condition_coerce_time() {
        // Arrange
        let (_db, conn, fx) = setup_coerce_fixture().await.unwrap();
        let fmap = field_map_coerce();
        let expr = odata_ast::Expr::Compare(
            Box::new(odata_ast::Expr::Identifier("t".to_owned())),
            odata_ast::CompareOperator::Eq,
            Box::new(odata_ast::Expr::Value(odata_ast::Value::Time(fx.t))),
        );

        // Act
        let cond = expr_to_condition::<ent_coerce::Entity>(&expr, &fmap).unwrap();
        let ids: Vec<i64> = ent_coerce::Entity::find()
            .filter(cond)
            .all(&conn)
            .await
            .unwrap()
            .into_iter()
            .map(|m| m.id)
            .collect();

        // Assert
        assert_eq!(ids, vec![1]);
    }

    #[test]
    fn expr_to_condition_coerce_type_mismatch_branches() {
        // Arrange
        let fmap = field_map_coerce();
        let u = Uuid::new_v4();
        let dt = chrono::DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let d = NaiveDate::from_ymd_opt(2024, 1, 2).unwrap();
        let t = NaiveTime::from_hms_opt(3, 4, 5).unwrap();

        // Act
        let mismatch_null = expr_to_condition::<ent_coerce::Entity>(
            &odata_ast::Expr::In(
                Box::new(odata_ast::Expr::Identifier("i".to_owned())),
                vec![odata_ast::Expr::Value(odata_ast::Value::Null)],
            ),
            &fmap,
        );
        let mismatch_string = expr_to_condition::<ent_coerce::Entity>(
            &odata_ast::Expr::Compare(
                Box::new(odata_ast::Expr::Identifier("i".to_owned())),
                odata_ast::CompareOperator::Eq,
                Box::new(odata_ast::Expr::Value(odata_ast::Value::String(
                    "x".to_owned(),
                ))),
            ),
            &fmap,
        );
        let mismatch_number = expr_to_condition::<ent_coerce::Entity>(
            &odata_ast::Expr::Compare(
                Box::new(odata_ast::Expr::Identifier("s".to_owned())),
                odata_ast::CompareOperator::Eq,
                Box::new(odata_ast::Expr::Value(odata_ast::Value::Number(
                    BigDecimal::from(1),
                ))),
            ),
            &fmap,
        );
        let mismatch_bool = expr_to_condition::<ent_coerce::Entity>(
            &odata_ast::Expr::Compare(
                Box::new(odata_ast::Expr::Identifier("s".to_owned())),
                odata_ast::CompareOperator::Eq,
                Box::new(odata_ast::Expr::Value(odata_ast::Value::Bool(true))),
            ),
            &fmap,
        );
        let mismatch_uuid = expr_to_condition::<ent_coerce::Entity>(
            &odata_ast::Expr::Compare(
                Box::new(odata_ast::Expr::Identifier("s".to_owned())),
                odata_ast::CompareOperator::Eq,
                Box::new(odata_ast::Expr::Value(odata_ast::Value::Uuid(u))),
            ),
            &fmap,
        );
        let mismatch_datetime = expr_to_condition::<ent_coerce::Entity>(
            &odata_ast::Expr::Compare(
                Box::new(odata_ast::Expr::Identifier("s".to_owned())),
                odata_ast::CompareOperator::Eq,
                Box::new(odata_ast::Expr::Value(odata_ast::Value::DateTime(dt))),
            ),
            &fmap,
        );
        let mismatch_date = expr_to_condition::<ent_coerce::Entity>(
            &odata_ast::Expr::Compare(
                Box::new(odata_ast::Expr::Identifier("s".to_owned())),
                odata_ast::CompareOperator::Eq,
                Box::new(odata_ast::Expr::Value(odata_ast::Value::Date(d))),
            ),
            &fmap,
        );
        let mismatch_time = expr_to_condition::<ent_coerce::Entity>(
            &odata_ast::Expr::Compare(
                Box::new(odata_ast::Expr::Identifier("s".to_owned())),
                odata_ast::CompareOperator::Eq,
                Box::new(odata_ast::Expr::Value(odata_ast::Value::Time(t))),
            ),
            &fmap,
        );
        let mismatch_i64_out_of_range_number = expr_to_condition::<ent_coerce::Entity>(
            &odata_ast::Expr::Compare(
                Box::new(odata_ast::Expr::Identifier("i".to_owned())),
                odata_ast::CompareOperator::Eq,
                Box::new(odata_ast::Expr::Value(odata_ast::Value::Number(
                    BigDecimal::from_str("9223372036854775808").unwrap(),
                ))),
            ),
            &fmap,
        );

        // Assert
        assert!(matches!(
            mismatch_null,
            Err(ODataBuildError::TypeMismatch {
                expected: FieldKind::I64,
                got: "null"
            })
        ));
        assert!(matches!(
            mismatch_string,
            Err(ODataBuildError::TypeMismatch {
                expected: FieldKind::I64,
                got: "string"
            })
        ));
        assert!(matches!(
            mismatch_number,
            Err(ODataBuildError::TypeMismatch {
                expected: FieldKind::String,
                got: "number"
            })
        ));
        assert!(matches!(
            mismatch_bool,
            Err(ODataBuildError::TypeMismatch {
                expected: FieldKind::String,
                got: "bool"
            })
        ));
        assert!(matches!(
            mismatch_uuid,
            Err(ODataBuildError::TypeMismatch {
                expected: FieldKind::String,
                got: "uuid"
            })
        ));
        assert!(matches!(
            mismatch_datetime,
            Err(ODataBuildError::TypeMismatch {
                expected: FieldKind::String,
                got: "datetime"
            })
        ));
        assert!(matches!(
            mismatch_date,
            Err(ODataBuildError::TypeMismatch {
                expected: FieldKind::String,
                got: "date"
            })
        ));
        assert!(matches!(
            mismatch_time,
            Err(ODataBuildError::TypeMismatch {
                expected: FieldKind::String,
                got: "time"
            })
        ));
        assert!(matches!(
            mismatch_i64_out_of_range_number,
            Err(ODataBuildError::TypeMismatch {
                expected: FieldKind::I64,
                got: "number"
            })
        ));
    }

    // ===== NULL handling =====

    #[tokio::test]
    async fn expr_compare_eq_null_rejected() {
        // Arrange
        let expr = odata_ast::Expr::Compare(
            Box::new(odata_ast::Expr::Identifier("name".to_owned())),
            odata_ast::CompareOperator::Eq,
            Box::new(odata_ast::Expr::Value(odata_ast::Value::Null)),
        );

        let fmap = field_map();

        // Act
        let result = expr_to_condition::<ent::Entity>(&expr, &fmap);

        // Assert: EQ null is actually supported (becomes IS NULL), so this should succeed
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn expr_compare_gt_null_rejected() {
        // Arrange
        let expr = odata_ast::Expr::Compare(
            Box::new(odata_ast::Expr::Identifier("name".to_owned())),
            odata_ast::CompareOperator::Gt,
            Box::new(odata_ast::Expr::Value(odata_ast::Value::Null)),
        );

        let fmap = field_map();

        // Act
        let result = expr_to_condition::<ent::Entity>(&expr, &fmap);

        // Assert
        assert!(matches!(
            result,
            Err(ODataBuildError::UnsupportedOp(
                odata_ast::CompareOperator::Gt
            ))
        ));
    }

    // ===== Error cases =====

    #[tokio::test]
    async fn expr_compare_field_to_field_rejected() {
        // Arrange
        let expr = odata_ast::Expr::Compare(
            Box::new(odata_ast::Expr::Identifier("name".to_owned())),
            odata_ast::CompareOperator::Eq,
            Box::new(odata_ast::Expr::Identifier("id".to_owned())),
        );

        let fmap = field_map();

        // Act
        let result = expr_to_condition::<ent::Entity>(&expr, &fmap);

        // Assert
        assert!(matches!(
            result,
            Err(ODataBuildError::Other(
                "field-to-field comparison is not supported"
            ))
        ));
    }

    #[tokio::test]
    async fn expr_compare_unknown_field_rejected() {
        // Arrange
        let expr = odata_ast::Expr::Compare(
            Box::new(odata_ast::Expr::Identifier("unknown".to_owned())),
            odata_ast::CompareOperator::Eq,
            Box::new(odata_ast::Expr::Value(odata_ast::Value::String(
                "test".to_owned(),
            ))),
        );

        let fmap = field_map();

        // Act
        let result = expr_to_condition::<ent::Entity>(&expr, &fmap);

        // Assert
        assert!(matches!(
            result,
            Err(ODataBuildError::UnknownField(f)) if f == "unknown"
        ));
    }

    // ===== IN operator =====

    #[tokio::test]
    async fn expr_in_matches_any_value() {
        // Arrange
        let (_db, conn) = setup_db().await.unwrap();
        create_schema(&conn).await.unwrap();
        seed(&conn, &[(1, "alice"), (2, "bob"), (3, "charlie")])
            .await
            .unwrap();

        let expr = odata_ast::Expr::In(
            Box::new(odata_ast::Expr::Identifier("name".to_owned())),
            vec![
                odata_ast::Expr::Value(odata_ast::Value::String("alice".to_owned())),
                odata_ast::Expr::Value(odata_ast::Value::String("charlie".to_owned())),
            ],
        );

        let fmap = field_map();

        // Act
        let cond = expr_to_condition::<ent::Entity>(&expr, &fmap).unwrap();
        let mut results = ent::Entity::find().filter(cond).all(&conn).await.unwrap();
        results.sort_by_key(|r| r.id);

        // Assert
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].name, "alice");
        assert_eq!(results[1].name, "charlie");
    }

    #[tokio::test]
    async fn expr_in_empty_list_matches_nothing() {
        // Arrange
        let (_db, conn) = setup_db().await.unwrap();
        create_schema(&conn).await.unwrap();
        seed(&conn, &[(1, "alice"), (2, "bob")]).await.unwrap();

        let expr = odata_ast::Expr::In(
            Box::new(odata_ast::Expr::Identifier("name".to_owned())),
            vec![],
        );

        let fmap = field_map();

        // Act
        let cond = expr_to_condition::<ent::Entity>(&expr, &fmap).unwrap();
        let results = ent::Entity::find().filter(cond).all(&conn).await.unwrap();

        // Assert
        assert_eq!(results.len(), 0, "Empty IN list should match nothing (1=0)");
    }

    #[tokio::test]
    async fn expr_in_non_literal_rejected() {
        // Arrange
        let expr = odata_ast::Expr::In(
            Box::new(odata_ast::Expr::Identifier("name".to_owned())),
            vec![odata_ast::Expr::Identifier("other".to_owned())],
        );

        let fmap = field_map();

        // Act
        let result = expr_to_condition::<ent::Entity>(&expr, &fmap);

        // Assert
        assert!(matches!(result, Err(ODataBuildError::NonLiteralInList)));
    }

    // ===== String functions =====

    #[tokio::test]
    async fn expr_function_contains_matches_substring() {
        // Arrange
        let (_db, conn) = setup_db().await.unwrap();
        create_schema(&conn).await.unwrap();
        seed(&conn, &[(1, "alice"), (2, "bob"), (3, "alicia")])
            .await
            .unwrap();

        let expr = odata_ast::Expr::Function(
            "contains".to_owned(),
            vec![
                odata_ast::Expr::Identifier("name".to_owned()),
                odata_ast::Expr::Value(odata_ast::Value::String("lic".to_owned())),
            ],
        );

        let fmap = field_map();

        // Act
        let cond = expr_to_condition::<ent::Entity>(&expr, &fmap).unwrap();
        let mut results = ent::Entity::find().filter(cond).all(&conn).await.unwrap();
        results.sort_by_key(|r| r.id);

        // Assert
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].name, "alice");
        assert_eq!(results[1].name, "alicia");
    }

    #[tokio::test]
    async fn expr_function_startswith_matches_prefix() {
        // Arrange
        let (_db, conn) = setup_db().await.unwrap();
        create_schema(&conn).await.unwrap();
        seed(&conn, &[(1, "alice"), (2, "bob"), (3, "alicia")])
            .await
            .unwrap();

        let expr = odata_ast::Expr::Function(
            "startswith".to_owned(),
            vec![
                odata_ast::Expr::Identifier("name".to_owned()),
                odata_ast::Expr::Value(odata_ast::Value::String("ali".to_owned())),
            ],
        );

        let fmap = field_map();

        // Act
        let cond = expr_to_condition::<ent::Entity>(&expr, &fmap).unwrap();
        let mut results = ent::Entity::find().filter(cond).all(&conn).await.unwrap();
        results.sort_by_key(|r| r.id);

        // Assert
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].name, "alice");
        assert_eq!(results[1].name, "alicia");
    }

    #[tokio::test]
    async fn expr_function_endswith_matches_suffix() {
        // Arrange
        let (_db, conn) = setup_db().await.unwrap();
        create_schema(&conn).await.unwrap();
        seed(&conn, &[(1, "alice"), (2, "bob"), (3, "charlie")])
            .await
            .unwrap();

        let expr = odata_ast::Expr::Function(
            "endswith".to_owned(),
            vec![
                odata_ast::Expr::Identifier("name".to_owned()),
                odata_ast::Expr::Value(odata_ast::Value::String("ie".to_owned())),
            ],
        );

        let fmap = field_map();

        // Act
        let cond = expr_to_condition::<ent::Entity>(&expr, &fmap).unwrap();
        let results = ent::Entity::find().filter(cond).all(&conn).await.unwrap();

        // Assert
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "charlie");
    }

    #[tokio::test]
    async fn expr_function_unsupported_rejected() {
        // Arrange
        let expr = odata_ast::Expr::Function(
            "substring".to_owned(),
            vec![
                odata_ast::Expr::Identifier("name".to_owned()),
                odata_ast::Expr::Value(odata_ast::Value::Number(BigDecimal::from(0))),
            ],
        );

        let fmap = field_map();

        // Act
        let result = expr_to_condition::<ent::Entity>(&expr, &fmap);

        // Assert
        assert!(matches!(
            result,
            Err(ODataBuildError::UnsupportedFn(f)) if f == "substring"
        ));
    }

    #[tokio::test]
    async fn expr_function_contains_on_non_string_field_rejected() {
        // Arrange
        let expr = odata_ast::Expr::Function(
            "contains".to_owned(),
            vec![
                odata_ast::Expr::Identifier("id".to_owned()),
                odata_ast::Expr::Value(odata_ast::Value::String("test".to_owned())),
            ],
        );

        let fmap = field_map();

        // Act
        let result = expr_to_condition::<ent::Entity>(&expr, &fmap);

        // Assert
        assert!(matches!(
            result,
            Err(ODataBuildError::TypeMismatch {
                expected: FieldKind::String,
                got: "non-string field"
            })
        ));
    }

    // ===== Bare expressions =====

    #[tokio::test]
    async fn expr_bare_identifier_rejected() {
        // Arrange
        let expr = odata_ast::Expr::Identifier("name".to_owned());
        let fmap = field_map();

        // Act
        let result = expr_to_condition::<ent::Entity>(&expr, &fmap);

        // Assert
        assert!(matches!(
            result,
            Err(ODataBuildError::BareIdentifier(f)) if f == "name"
        ));
    }

    #[tokio::test]
    async fn expr_bare_literal_rejected() {
        // Arrange
        let expr = odata_ast::Expr::Value(odata_ast::Value::String("test".to_owned()));
        let fmap = field_map();

        // Act
        let result = expr_to_condition::<ent::Entity>(&expr, &fmap);

        // Assert
        assert!(matches!(result, Err(ODataBuildError::BareLiteral)));
    }
}

#[cfg(feature = "sqlite")]
mod paginate_with_odata_sqlite {
    use super::*;
    use crate::{odata::core::paginate_with_odata, odata::LimitCfg, ConnectOpts, DbHandle, Result};
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

    async fn setup_db() -> Result<(DbHandle, DatabaseConnection)> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dsn = format!("sqlite:file:paginate_with_odata_{now}?mode=memory&cache=shared");
        let db = DbHandle::connect(&dsn, ConnectOpts::default()).await?;
        let conn = db.sea_secure().conn().clone();
        Ok((db, conn))
    }

    async fn create_schema(conn: &DatabaseConnection) -> Result<()> {
        conn.execute_unprepared(
            r#"CREATE TABLE paginate_test (
                    id INTEGER PRIMARY KEY NOT NULL,
                    name TEXT NOT NULL,
                    score INTEGER NOT NULL
                )"#,
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
        let (_db, conn) = setup_db().await.unwrap();
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
        let (_db, conn) = setup_db().await.unwrap();
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
        let (_db, conn) = setup_db().await.unwrap();
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
        let (_db, conn) = setup_db().await.unwrap();
        create_schema(&conn).await.unwrap();
        seed(&conn, &[(1, "alice", 10), (2, "bob", 20), (3, "charlie", 30)])
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
        let (_db, conn) = setup_db().await.unwrap();
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
}

#[cfg(feature = "sqlite")]
mod apply_ext_methods_sqlite {
    use super::*;
    use crate::{
        odata::core::{CursorApplyExt, ODataExt, ODataOrderExt, ODataQueryExt},
        ConnectOpts, DbHandle, Result,
    };
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

    async fn setup_db() -> Result<(DbHandle, DatabaseConnection)> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dsn = format!("sqlite:file:apply_ext_{now}?mode=memory&cache=shared");
        let db = DbHandle::connect(&dsn, ConnectOpts::default()).await?;
        let conn = db.sea_secure().conn().clone();
        Ok((db, conn))
    }

    async fn create_schema(conn: &DatabaseConnection) -> Result<()> {
        conn.execute_unprepared(
            r#"CREATE TABLE apply_ext_test (
                    id INTEGER PRIMARY KEY NOT NULL,
                    name TEXT NOT NULL,
                    score INTEGER NOT NULL
                )"#,
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
        let (_db, conn) = setup_db().await.unwrap();
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
        let (_db, conn) = setup_db().await.unwrap();
        create_schema(&conn).await.unwrap();
        seed(
            &conn,
            &[
                (1, "charlie", 30),
                (2, "alice", 10),
                (3, "bob", 20),
            ],
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
        let (_db, conn) = setup_db().await.unwrap();
        create_schema(&conn).await.unwrap();
        seed(
            &conn,
            &[
                (1, "alice", 10),
                (2, "bob", 20),
                (3, "charlie", 30),
            ],
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
        let (_db, conn) = setup_db().await.unwrap();
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
        let (_db, conn) = setup_db().await.unwrap();
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
        let (_db, conn) = setup_db().await.unwrap();
        create_schema(&conn).await.unwrap();
        seed(
            &conn,
            &[
                (1, "alice", 20),
                (2, "bob", 20),
                (3, "charlie", 20),
            ],
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
}
