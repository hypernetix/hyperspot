//! Unit tests for core `OData` functions (legacy `FieldMap`-based system)

use crate::odata::core::{
    build_cursor_predicate, ensure_tiebreaker, expr_to_condition, ODataBuildError,
};
use crate::odata::{FieldKind, FieldMap};
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
    let order = ODataOrderBy(vec![OrderKey {
        field: "name".to_owned(),
        dir: SortDir::Asc,
    }]);

    let order = ensure_tiebreaker(order, "id", SortDir::Desc);

    assert_eq!(order.0.len(), 2);
    assert_eq!(order.0[0].field, "name");
    assert_eq!(order.0[0].dir, SortDir::Asc);
    assert_eq!(order.0[1].field, "id");
    assert_eq!(order.0[1].dir, SortDir::Desc);
}

#[test]
fn ensure_tiebreaker_does_not_duplicate_when_present() {
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

    let order = ensure_tiebreaker(order, "id", SortDir::Asc);

    assert_eq!(order.0.len(), 2);
    assert_eq!(order.0[1].field, "id");
    assert_eq!(order.0[1].dir, SortDir::Desc);
}

#[test]
fn build_cursor_predicate_rejects_key_count_mismatch() {
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

    let err = build_cursor_predicate::<Entity>(&cursor, &order, &fmap).unwrap_err();

    assert!(matches!(
        err,
        ODataBuildError::Other("cursor keys count mismatch with order fields")
    ));
}

#[test]
fn build_cursor_predicate_rejects_unknown_order_field() {
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

    let err = build_cursor_predicate::<Entity>(&cursor, &order, &fmap).unwrap_err();

    assert!(matches!(err, ODataBuildError::UnknownField(f) if f == "unknown"));
}

#[test]
fn expr_to_condition_rejects_bare_identifier() {
    let expr = Expr::Identifier("name".to_owned());
    let fmap = field_map();

    let err = expr_to_condition::<Entity>(&expr, &fmap).unwrap_err();

    assert!(matches!(err, ODataBuildError::BareIdentifier(f) if f == "name"));
}

#[test]
fn expr_to_condition_rejects_bare_literal() {
    let expr = Expr::Value(Value::String("x".to_owned()));
    let fmap = field_map();

    let err = expr_to_condition::<Entity>(&expr, &fmap).unwrap_err();

    assert!(matches!(err, ODataBuildError::BareLiteral));
}

#[test]
fn expr_to_condition_rejects_field_to_field_comparison() {
    let expr = Expr::Compare(
        Box::new(Expr::Identifier("name".to_owned())),
        CompareOperator::Eq,
        Box::new(Expr::Identifier("email".to_owned())),
    );
    let fmap = field_map();

    let err = expr_to_condition::<Entity>(&expr, &fmap).unwrap_err();

    assert!(matches!(
        err,
        ODataBuildError::Other("field-to-field comparison is not supported")
    ));
}

#[test]
fn expr_to_condition_allows_eq_null() {
    let expr = Expr::Compare(
        Box::new(Expr::Identifier("email".to_owned())),
        CompareOperator::Eq,
        Box::new(Expr::Value(Value::Null)),
    );
    let fmap = field_map();

    let cond = expr_to_condition::<Entity>(&expr, &fmap).unwrap();

    assert!(!cond.is_empty());
}

#[test]
fn expr_to_condition_rejects_non_equality_null_comparison() {
    let expr = Expr::Compare(
        Box::new(Expr::Identifier("email".to_owned())),
        CompareOperator::Gt,
        Box::new(Expr::Value(Value::Null)),
    );
    let fmap = field_map();

    let err = expr_to_condition::<Entity>(&expr, &fmap).unwrap_err();

    assert!(matches!(
        err,
        ODataBuildError::UnsupportedOp(CompareOperator::Gt)
    ));
}

#[test]
fn expr_to_condition_rejects_in_with_non_literal_list_items() {
    let expr = Expr::In(
        Box::new(Expr::Identifier("score".to_owned())),
        vec![Expr::Identifier("score".to_owned())],
    );
    let fmap = field_map();

    let err = expr_to_condition::<Entity>(&expr, &fmap).unwrap_err();

    assert!(matches!(err, ODataBuildError::NonLiteralInList));
}

#[test]
fn expr_to_condition_translates_empty_in_list_to_deny_all() {
    let expr = Expr::In(Box::new(Expr::Identifier("score".to_owned())), vec![]);
    let fmap = field_map();

    let cond = expr_to_condition::<Entity>(&expr, &fmap).unwrap();

    assert!(!cond.is_empty());
}

#[test]
fn expr_to_condition_rejects_type_mismatch_for_field_kind() {
    let expr = Expr::Compare(
        Box::new(Expr::Identifier("score".to_owned())),
        CompareOperator::Eq,
        Box::new(Expr::Value(Value::String("not-a-number".to_owned()))),
    );
    let fmap = field_map();

    let err = expr_to_condition::<Entity>(&expr, &fmap).unwrap_err();

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
    let id = uuid::Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap();
    let value = sea_orm::Value::Uuid(Some(Box::new(id)));

    let encoded =
        crate::odata::sea_orm_filter::encode_cursor_value(&value, FieldKind::Uuid).unwrap();
    let decoded =
        crate::odata::sea_orm_filter::parse_cursor_value(FieldKind::Uuid, &encoded).unwrap();

    assert_eq!(encoded, "123e4567-e89b-12d3-a456-426614174000");
    assert!(matches!(decoded, sea_orm::Value::Uuid(Some(v)) if *v == id));
}

#[test]
fn typed_cursor_value_f64_round_trip() {
    let value = sea_orm::Value::Double(Some(3.5));

    let encoded =
        crate::odata::sea_orm_filter::encode_cursor_value(&value, FieldKind::F64).unwrap();
    let decoded =
        crate::odata::sea_orm_filter::parse_cursor_value(FieldKind::F64, &encoded).unwrap();

    assert_eq!(encoded, "3.5");
    assert!(matches!(decoded, sea_orm::Value::Double(Some(v)) if (v - 3.5).abs() < 1e-12));
}
