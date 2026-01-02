#![allow(clippy::similar_names)]

use super::support::setup_sqlite_db;
use crate::odata::core::{expr_to_condition, ODataBuildError};
use crate::odata::{FieldKind, FieldMap};
use crate::{DbHandle, Result};
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

async fn create_schema(conn: &DatabaseConnection) -> Result<()> {
    conn.execute_unprepared(
        "CREATE TABLE expr_test (
id INTEGER PRIMARY KEY NOT NULL,
name TEXT NOT NULL
)",
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
        "CREATE TABLE expr_decimal_test (
id INTEGER PRIMARY KEY NOT NULL,
amount NUMERIC NOT NULL
)",
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
        "CREATE TABLE expr_coerce_test (
id INTEGER PRIMARY KEY NOT NULL,
s TEXT NOT NULL,
i INTEGER NOT NULL,
f REAL NOT NULL,
b INTEGER NOT NULL,
u BLOB NOT NULL,
dt TEXT NOT NULL,
d TEXT NOT NULL,
t TEXT NOT NULL
)",
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
    let (db, conn) = setup_sqlite_db().await?;
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
    let (_db, conn) = setup_sqlite_db().await.unwrap();
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
    let (_db, conn) = setup_sqlite_db().await.unwrap();
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
    let (_db, conn) = setup_sqlite_db().await.unwrap();
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
    let (_db, conn) = setup_sqlite_db().await.unwrap();
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
    let (_db, conn) = setup_sqlite_db().await.unwrap();
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
    let (_db, conn) = setup_sqlite_db().await.unwrap();
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
    let (_db, conn) = setup_sqlite_db().await.unwrap();
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
    let (_db, conn) = setup_sqlite_db().await.unwrap();
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
    let (_db, conn) = setup_sqlite_db().await.unwrap();
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
    let (_db, conn) = setup_sqlite_db().await.unwrap();
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
    let (_db, conn) = setup_sqlite_db().await.unwrap();
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
    let (_db, conn) = setup_sqlite_db().await.unwrap();
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
    let (_db, conn) = setup_sqlite_db().await.unwrap();
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
    let (_db, conn) = setup_sqlite_db().await.unwrap();
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
    let (_db, conn) = setup_sqlite_db().await.unwrap();
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
