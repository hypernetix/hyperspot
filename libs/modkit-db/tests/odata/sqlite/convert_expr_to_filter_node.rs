use super::support::setup_sqlite_db;
use crate::odata::filter::convert_expr_to_filter_node;
use crate::odata::sea_orm_filter::{filter_node_to_condition, FieldToColumn, ODataFieldMapping};
use crate::odata::FieldKind;
use crate::Result;
use anyhow::anyhow;
use bigdecimal::BigDecimal;
use modkit_odata::ast as odata_ast;
use sea_orm::{ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::str::FromStr;

mod ent {
    use sea_orm::entity::prelude::*;

    #[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "filter_node_test")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub name: String,
        pub score: i64,
        pub active: bool,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TestFilterField {
    Id,
    Name,
    Score,
    Active,
}

impl crate::odata::filter::FilterField for TestFilterField {
    const FIELDS: &'static [Self] = &[Self::Id, Self::Name, Self::Score, Self::Active];

    fn from_name(name: &str) -> Option<Self> {
        match name {
            "id" => Some(Self::Id),
            "name" => Some(Self::Name),
            "score" => Some(Self::Score),
            "active" => Some(Self::Active),
            _ => None,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::Name => "name",
            Self::Score => "score",
            Self::Active => "active",
        }
    }

    fn kind(&self) -> FieldKind {
        match self {
            Self::Id | Self::Score => FieldKind::I64,
            Self::Name => FieldKind::String,
            Self::Active => FieldKind::Bool,
        }
    }
}

struct TestMapper;

impl FieldToColumn<TestFilterField> for TestMapper {
    type Column = ent::Column;

    fn map_field(field: TestFilterField) -> Self::Column {
        match field {
            TestFilterField::Id => ent::Column::Id,
            TestFilterField::Name => ent::Column::Name,
            TestFilterField::Score => ent::Column::Score,
            TestFilterField::Active => ent::Column::Active,
        }
    }
}

impl ODataFieldMapping<TestFilterField> for TestMapper {
    type Entity = ent::Entity;

    fn extract_cursor_value(_model: &ent::Model, _field: TestFilterField) -> sea_orm::Value {
        unimplemented!("cursor extraction not needed for filter tests")
    }
}

async fn create_schema(conn: &DatabaseConnection) -> Result<()> {
    conn.execute_unprepared(
        "CREATE TABLE filter_node_test (
id INTEGER PRIMARY KEY NOT NULL,
name TEXT NOT NULL,
score INTEGER NOT NULL,
active INTEGER NOT NULL
)",
    )
    .await
    .map_err(|e| crate::DbError::Other(anyhow!(e.to_string())))?;
    Ok(())
}

async fn seed(conn: &DatabaseConnection, rows: &[(i64, &str, i64, bool)]) -> Result<()> {
    for (id, name, score, active) in rows {
        ent::Entity::insert(ent::ActiveModel {
            id: Set(*id),
            name: Set((*name).to_owned()),
            score: Set(*score),
            active: Set(*active),
        })
        .exec(conn)
        .await
        .map_err(|e| crate::DbError::Other(anyhow!(e.to_string())))?;
    }
    Ok(())
}

#[tokio::test]
async fn convert_expr_compare_eq() {
    let (_db, conn) = setup_sqlite_db().await.unwrap();
    create_schema(&conn).await.unwrap();
    seed(
        &conn,
        &[
            (1, "alice", 10, true),
            (2, "bob", 20, false),
            (3, "charlie", 30, true),
        ],
    )
    .await
    .unwrap();

    let expr = odata_ast::Expr::Compare(
        Box::new(odata_ast::Expr::Identifier("name".to_owned())),
        odata_ast::CompareOperator::Eq,
        Box::new(odata_ast::Expr::Value(odata_ast::Value::String(
            "bob".to_owned(),
        ))),
    );

    let filter_node = convert_expr_to_filter_node::<TestFilterField>(&expr).unwrap();
    let condition = filter_node_to_condition::<TestFilterField, TestMapper>(&filter_node).unwrap();

    let results: Vec<String> = ent::Entity::find()
        .filter(condition)
        .all(&conn)
        .await
        .unwrap()
        .into_iter()
        .map(|m| m.name)
        .collect();

    assert_eq!(results, vec!["bob"]);
}

#[tokio::test]
async fn convert_expr_compare_gt() {
    let (_db, conn) = setup_sqlite_db().await.unwrap();
    create_schema(&conn).await.unwrap();
    seed(
        &conn,
        &[
            (1, "alice", 10, true),
            (2, "bob", 20, false),
            (3, "charlie", 30, true),
        ],
    )
    .await
    .unwrap();

    let expr = odata_ast::Expr::Compare(
        Box::new(odata_ast::Expr::Identifier("score".to_owned())),
        odata_ast::CompareOperator::Gt,
        Box::new(odata_ast::Expr::Value(odata_ast::Value::Number(
            bigdecimal::BigDecimal::from(15),
        ))),
    );

    let filter_node = convert_expr_to_filter_node::<TestFilterField>(&expr).unwrap();
    let condition = filter_node_to_condition::<TestFilterField, TestMapper>(&filter_node).unwrap();

    let results: Vec<i64> = ent::Entity::find()
        .filter(condition)
        .all(&conn)
        .await
        .unwrap()
        .into_iter()
        .map(|m| m.id)
        .collect();

    assert_eq!(results, vec![2, 3]);
}

#[tokio::test]
async fn convert_expr_and_combines_conditions() {
    let (_db, conn) = setup_sqlite_db().await.unwrap();
    create_schema(&conn).await.unwrap();
    seed(
        &conn,
        &[
            (1, "alice", 10, true),
            (2, "bob", 20, false),
            (3, "charlie", 30, true),
        ],
    )
    .await
    .unwrap();

    let expr = odata_ast::Expr::And(
        Box::new(odata_ast::Expr::Compare(
            Box::new(odata_ast::Expr::Identifier("score".to_owned())),
            odata_ast::CompareOperator::Ge,
            Box::new(odata_ast::Expr::Value(odata_ast::Value::Number(
                bigdecimal::BigDecimal::from(10),
            ))),
        )),
        Box::new(odata_ast::Expr::Compare(
            Box::new(odata_ast::Expr::Identifier("active".to_owned())),
            odata_ast::CompareOperator::Eq,
            Box::new(odata_ast::Expr::Value(odata_ast::Value::Bool(true))),
        )),
    );

    let filter_node = convert_expr_to_filter_node::<TestFilterField>(&expr).unwrap();
    let condition = filter_node_to_condition::<TestFilterField, TestMapper>(&filter_node).unwrap();

    let results: Vec<String> = ent::Entity::find()
        .filter(condition)
        .all(&conn)
        .await
        .unwrap()
        .into_iter()
        .map(|m| m.name)
        .collect();

    assert_eq!(results, vec!["alice", "charlie"]);
}

#[tokio::test]
async fn convert_expr_or_matches_either() {
    let (_db, conn) = setup_sqlite_db().await.unwrap();
    create_schema(&conn).await.unwrap();
    seed(
        &conn,
        &[
            (1, "alice", 10, true),
            (2, "bob", 20, false),
            (3, "charlie", 30, true),
        ],
    )
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
                "charlie".to_owned(),
            ))),
        )),
    );

    let filter_node = convert_expr_to_filter_node::<TestFilterField>(&expr).unwrap();
    let condition = filter_node_to_condition::<TestFilterField, TestMapper>(&filter_node).unwrap();

    let results: Vec<String> = ent::Entity::find()
        .filter(condition)
        .all(&conn)
        .await
        .unwrap()
        .into_iter()
        .map(|m| m.name)
        .collect();

    assert_eq!(results, vec!["alice", "charlie"]);
}

#[tokio::test]
async fn convert_expr_not_inverts_condition() {
    let (_db, conn) = setup_sqlite_db().await.unwrap();
    create_schema(&conn).await.unwrap();
    seed(
        &conn,
        &[
            (1, "alice", 10, true),
            (2, "bob", 20, false),
            (3, "charlie", 30, true),
        ],
    )
    .await
    .unwrap();

    let expr = odata_ast::Expr::Not(Box::new(odata_ast::Expr::Compare(
        Box::new(odata_ast::Expr::Identifier("active".to_owned())),
        odata_ast::CompareOperator::Eq,
        Box::new(odata_ast::Expr::Value(odata_ast::Value::Bool(true))),
    )));

    let filter_node = convert_expr_to_filter_node::<TestFilterField>(&expr).unwrap();
    let condition = filter_node_to_condition::<TestFilterField, TestMapper>(&filter_node).unwrap();

    let results: Vec<String> = ent::Entity::find()
        .filter(condition)
        .all(&conn)
        .await
        .unwrap()
        .into_iter()
        .map(|m| m.name)
        .collect();

    assert_eq!(results, vec!["bob"]);
}

#[tokio::test]
async fn convert_expr_function_contains() {
    let (_db, conn) = setup_sqlite_db().await.unwrap();
    create_schema(&conn).await.unwrap();
    seed(
        &conn,
        &[
            (1, "alice", 10, true),
            (2, "bob", 20, false),
            (3, "charlie", 30, true),
        ],
    )
    .await
    .unwrap();

    let expr = odata_ast::Expr::Function(
        "contains".to_owned(),
        vec![
            odata_ast::Expr::Identifier("name".to_owned()),
            odata_ast::Expr::Value(odata_ast::Value::String("li".to_owned())),
        ],
    );

    let filter_node = convert_expr_to_filter_node::<TestFilterField>(&expr).unwrap();
    let condition = filter_node_to_condition::<TestFilterField, TestMapper>(&filter_node).unwrap();

    let results: Vec<String> = ent::Entity::find()
        .filter(condition)
        .all(&conn)
        .await
        .unwrap()
        .into_iter()
        .map(|m| m.name)
        .collect();

    assert_eq!(results, vec!["alice", "charlie"]);
}

#[tokio::test]
async fn convert_expr_function_startswith() {
    let (_db, conn) = setup_sqlite_db().await.unwrap();
    create_schema(&conn).await.unwrap();
    seed(
        &conn,
        &[
            (1, "alice", 10, true),
            (2, "bob", 20, false),
            (3, "charlie", 30, true),
        ],
    )
    .await
    .unwrap();

    let expr = odata_ast::Expr::Function(
        "startswith".to_owned(),
        vec![
            odata_ast::Expr::Identifier("name".to_owned()),
            odata_ast::Expr::Value(odata_ast::Value::String("ch".to_owned())),
        ],
    );

    let filter_node = convert_expr_to_filter_node::<TestFilterField>(&expr).unwrap();
    let condition = filter_node_to_condition::<TestFilterField, TestMapper>(&filter_node).unwrap();

    let results: Vec<String> = ent::Entity::find()
        .filter(condition)
        .all(&conn)
        .await
        .unwrap()
        .into_iter()
        .map(|m| m.name)
        .collect();

    assert_eq!(results, vec!["charlie"]);
}

#[tokio::test]
async fn convert_expr_function_endswith() {
    let (_db, conn) = setup_sqlite_db().await.unwrap();
    create_schema(&conn).await.unwrap();
    seed(
        &conn,
        &[
            (1, "alice", 10, true),
            (2, "bob", 20, false),
            (3, "charlie", 30, true),
        ],
    )
    .await
    .unwrap();

    let expr = odata_ast::Expr::Function(
        "endswith".to_owned(),
        vec![
            odata_ast::Expr::Identifier("name".to_owned()),
            odata_ast::Expr::Value(odata_ast::Value::String("ce".to_owned())),
        ],
    );

    let filter_node = convert_expr_to_filter_node::<TestFilterField>(&expr).unwrap();
    let condition = filter_node_to_condition::<TestFilterField, TestMapper>(&filter_node).unwrap();

    let results: Vec<String> = ent::Entity::find()
        .filter(condition)
        .all(&conn)
        .await
        .unwrap()
        .into_iter()
        .map(|m| m.name)
        .collect();

    assert_eq!(results, vec!["alice"]);
}

#[tokio::test]
async fn convert_expr_unknown_field_error() {
    let expr = odata_ast::Expr::Compare(
        Box::new(odata_ast::Expr::Identifier("unknown".to_owned())),
        odata_ast::CompareOperator::Eq,
        Box::new(odata_ast::Expr::Value(odata_ast::Value::String(
            "test".to_owned(),
        ))),
    );

    let result = convert_expr_to_filter_node::<TestFilterField>(&expr);
    assert!(matches!(
    result,
    Err(crate::odata::filter::FilterError::UnknownField(ref f)) if f == "unknown"
    ));
}

#[tokio::test]
async fn convert_expr_bare_identifier_error() {
    let expr = odata_ast::Expr::Identifier("name".to_owned());

    let result = convert_expr_to_filter_node::<TestFilterField>(&expr);
    assert!(matches!(
    result,
    Err(crate::odata::filter::FilterError::BareIdentifier(ref f)) if f == "name"
    ));
}

#[tokio::test]
async fn convert_expr_field_to_field_comparison_error() {
    let expr = odata_ast::Expr::Compare(
        Box::new(odata_ast::Expr::Identifier("name".to_owned())),
        odata_ast::CompareOperator::Eq,
        Box::new(odata_ast::Expr::Identifier("score".to_owned())),
    );

    let result = convert_expr_to_filter_node::<TestFilterField>(&expr);
    assert!(matches!(
        result,
        Err(crate::odata::filter::FilterError::FieldToFieldComparison)
    ));
}

#[tokio::test]
async fn convert_expr_ne_operator() {
    let (_db, conn) = setup_sqlite_db().await.unwrap();
    create_schema(&conn).await.unwrap();
    seed(&conn, &[(1, "alice", 10, true), (2, "bob", 20, false)])
        .await
        .unwrap();

    let expr = odata_ast::Expr::Compare(
        Box::new(odata_ast::Expr::Identifier("name".to_owned())),
        odata_ast::CompareOperator::Ne,
        Box::new(odata_ast::Expr::Value(odata_ast::Value::String(
            "alice".to_owned(),
        ))),
    );

    let filter_node = convert_expr_to_filter_node::<TestFilterField>(&expr).unwrap();
    let condition = filter_node_to_condition::<TestFilterField, TestMapper>(&filter_node).unwrap();

    let results: Vec<String> = ent::Entity::find()
        .filter(condition)
        .all(&conn)
        .await
        .unwrap()
        .into_iter()
        .map(|m| m.name)
        .collect();

    assert_eq!(results, vec!["bob"]);
}

#[tokio::test]
async fn convert_expr_lt_operator() {
    let (_db, conn) = setup_sqlite_db().await.unwrap();
    create_schema(&conn).await.unwrap();
    seed(
        &conn,
        &[
            (1, "alice", 10, true),
            (2, "bob", 20, false),
            (3, "charlie", 15, true),
        ],
    )
    .await
    .unwrap();

    let expr = odata_ast::Expr::Compare(
        Box::new(odata_ast::Expr::Identifier("score".to_owned())),
        odata_ast::CompareOperator::Lt,
        Box::new(odata_ast::Expr::Value(odata_ast::Value::Number(
            BigDecimal::from_str("20").unwrap(),
        ))),
    );

    let filter_node = convert_expr_to_filter_node::<TestFilterField>(&expr).unwrap();
    let condition = filter_node_to_condition::<TestFilterField, TestMapper>(&filter_node).unwrap();

    let results: Vec<i64> = ent::Entity::find()
        .filter(condition)
        .all(&conn)
        .await
        .unwrap()
        .into_iter()
        .map(|m| m.score)
        .collect();

    assert_eq!(results, vec![10, 15]);
}

#[tokio::test]
async fn convert_expr_le_operator() {
    let (_db, conn) = setup_sqlite_db().await.unwrap();
    create_schema(&conn).await.unwrap();
    seed(
        &conn,
        &[
            (1, "alice", 10, true),
            (2, "bob", 20, false),
            (3, "charlie", 15, true),
        ],
    )
    .await
    .unwrap();

    let expr = odata_ast::Expr::Compare(
        Box::new(odata_ast::Expr::Identifier("score".to_owned())),
        odata_ast::CompareOperator::Le,
        Box::new(odata_ast::Expr::Value(odata_ast::Value::Number(
            BigDecimal::from_str("15").unwrap(),
        ))),
    );

    let filter_node = convert_expr_to_filter_node::<TestFilterField>(&expr).unwrap();
    let condition = filter_node_to_condition::<TestFilterField, TestMapper>(&filter_node).unwrap();

    let results: Vec<i64> = ent::Entity::find()
        .filter(condition)
        .all(&conn)
        .await
        .unwrap()
        .into_iter()
        .map(|m| m.score)
        .collect();

    assert_eq!(results, vec![10, 15]);
}

#[tokio::test]
async fn convert_expr_startswith_non_string_field_error() {
    let expr = odata_ast::Expr::Function(
        "startswith".to_owned(),
        vec![
            odata_ast::Expr::Identifier("score".to_owned()),
            odata_ast::Expr::Value(odata_ast::Value::String("10".to_owned())),
        ],
    );

    let result = convert_expr_to_filter_node::<TestFilterField>(&expr);
    assert!(matches!(
        result,
        Err(crate::odata::filter::FilterError::TypeMismatch {
            field: ref f,
            expected: FieldKind::String,
            got: ref g
        }) if f == "score" && g == "non-string"
    ));
}

#[tokio::test]
async fn convert_expr_endswith_non_string_field_error() {
    let expr = odata_ast::Expr::Function(
        "endswith".to_owned(),
        vec![
            odata_ast::Expr::Identifier("active".to_owned()),
            odata_ast::Expr::Value(odata_ast::Value::String("true".to_owned())),
        ],
    );

    let result = convert_expr_to_filter_node::<TestFilterField>(&expr);
    assert!(matches!(
        result,
        Err(crate::odata::filter::FilterError::TypeMismatch {
            field: ref f,
            expected: FieldKind::String,
            got: ref g
        }) if f == "active" && g == "non-string"
    ));
}

#[tokio::test]
async fn convert_expr_contains_non_string_field_error() {
    let expr = odata_ast::Expr::Function(
        "contains".to_owned(),
        vec![
            odata_ast::Expr::Identifier("id".to_owned()),
            odata_ast::Expr::Value(odata_ast::Value::String("1".to_owned())),
        ],
    );

    let result = convert_expr_to_filter_node::<TestFilterField>(&expr);
    assert!(matches!(
        result,
        Err(crate::odata::filter::FilterError::TypeMismatch {
            field: ref f,
            expected: FieldKind::String,
            got: ref g
        }) if f == "id" && g == "non-string"
    ));
}
