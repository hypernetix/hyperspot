//! Unit tests for core OData functions (legacy FieldMap-based)

use modkit_db::odata::{
    build_cursor_for_model, parse_cursor_value, FieldKind, FieldMap, ODataBuildError,
};
use modkit_odata::{Error as ODataError, ODataOrderBy, OrderKey, SortDir};
use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "test_extractor")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[test]
fn field_map_insert_with_extractor_uses_custom_cursor_encoding() {
    // Arrange
    fn custom_name_extractor(m: &Model) -> String {
        format!("custom_{}", m.name.to_uppercase())
    }

    let fmap = FieldMap::<Entity>::new()
        .insert("id", Column::Id, FieldKind::I64)
        .insert_with_extractor(
            "name",
            Column::Name,
            FieldKind::String,
            custom_name_extractor,
        );

    let model = Model {
        id: 42,
        name: "test".to_owned(),
    };

    // Act
    let encoded_id = fmap.encode_model_key(&model, "id");
    let encoded_name = fmap.encode_model_key(&model, "name");

    // Assert
    assert_eq!(
        encoded_id, None,
        "id field has no extractor, should return None"
    );
    assert_eq!(
        encoded_name,
        Some("custom_TEST".to_owned()),
        "name field should use custom extractor"
    );
}

#[test]
fn field_map_insert_without_extractor_returns_none_for_encode_model_key() {
    // Arrange
    let fmap = FieldMap::<Entity>::new().insert("name", Column::Name, FieldKind::String);

    let model = Model {
        id: 1,
        name: "test".to_owned(),
    };

    // Act
    let result = fmap.encode_model_key(&model, "name");

    // Assert
    assert_eq!(result, None, "field without extractor should return None");
}

#[test]
fn parse_cursor_value_string() {
    let result = parse_cursor_value(FieldKind::String, "hello").unwrap();
    let sea_orm::Value::String(Some(s)) = result else {
        panic!("expected String value");
    };
    assert_eq!(*s, "hello");
}

#[test]
fn parse_cursor_value_i64_success() {
    let result = parse_cursor_value(FieldKind::I64, "42").unwrap();
    assert!(matches!(result, sea_orm::Value::BigInt(Some(42))));
}

#[test]
fn parse_cursor_value_i64_invalid() {
    let result = parse_cursor_value(FieldKind::I64, "not_a_number");
    assert!(matches!(
        result,
        Err(ODataBuildError::Other("invalid i64 in cursor"))
    ));
}

#[test]
fn parse_cursor_value_f64_success() {
    let result = parse_cursor_value(FieldKind::F64, "3.14").unwrap();
    let sea_orm::Value::Double(Some(f)) = result else {
        panic!("expected Double value");
    };
    #[allow(clippy::approx_constant)]
    {
        assert!((f - 3.14).abs() < 1e-10);
    }
}

#[test]
fn parse_cursor_value_f64_invalid() {
    let result = parse_cursor_value(FieldKind::F64, "not_a_float");
    assert!(matches!(
        result,
        Err(ODataBuildError::Other("invalid f64 in cursor"))
    ));
}

#[test]
fn parse_cursor_value_bool_success() {
    let result = parse_cursor_value(FieldKind::Bool, "true").unwrap();
    assert!(matches!(result, sea_orm::Value::Bool(Some(true))));
}

#[test]
fn parse_cursor_value_bool_invalid() {
    let result = parse_cursor_value(FieldKind::Bool, "not_a_bool");
    assert!(matches!(
        result,
        Err(ODataBuildError::Other("invalid bool in cursor"))
    ));
}

#[test]
fn parse_cursor_value_uuid_success() {
    let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
    let result = parse_cursor_value(FieldKind::Uuid, uuid_str).unwrap();
    let sea_orm::Value::Uuid(Some(u)) = result else {
        panic!("expected Uuid value");
    };
    assert_eq!(u.to_string(), uuid_str);
}

#[test]
fn parse_cursor_value_uuid_invalid() {
    let result = parse_cursor_value(FieldKind::Uuid, "not-a-uuid");
    assert!(matches!(
        result,
        Err(ODataBuildError::Other("invalid uuid in cursor"))
    ));
}

#[test]
fn parse_cursor_value_datetime_utc_success() {
    use chrono::Datelike;
    let dt_str = "2024-01-01T12:00:00Z";
    let result = parse_cursor_value(FieldKind::DateTimeUtc, dt_str).unwrap();
    let sea_orm::Value::ChronoDateTimeUtc(Some(dt)) = result else {
        panic!("expected ChronoDateTimeUtc value");
    };
    assert_eq!(dt.year(), 2024);
    assert_eq!(dt.month(), 1);
    assert_eq!(dt.day(), 1);
}

#[test]
fn parse_cursor_value_datetime_utc_invalid() {
    let result = parse_cursor_value(FieldKind::DateTimeUtc, "not-a-datetime");
    assert!(matches!(
        result,
        Err(ODataBuildError::Other("invalid datetime in cursor"))
    ));
}

#[test]
fn parse_cursor_value_date_success() {
    use chrono::Datelike;
    let date_str = "2024-01-15";
    let result = parse_cursor_value(FieldKind::Date, date_str).unwrap();
    let sea_orm::Value::ChronoDate(Some(d)) = result else {
        panic!("expected ChronoDate value");
    };
    assert_eq!(d.year(), 2024);
    assert_eq!(d.month(), 1);
    assert_eq!(d.day(), 15);
}

#[test]
fn parse_cursor_value_date_invalid() {
    let result = parse_cursor_value(FieldKind::Date, "not-a-date");
    assert!(matches!(
        result,
        Err(ODataBuildError::Other("invalid date in cursor"))
    ));
}

#[test]
fn parse_cursor_value_time_success() {
    use chrono::Timelike;
    let time_str = "14:30:45";
    let result = parse_cursor_value(FieldKind::Time, time_str).unwrap();
    let sea_orm::Value::ChronoTime(Some(t)) = result else {
        panic!("expected ChronoTime value");
    };
    assert_eq!(t.hour(), 14);
    assert_eq!(t.minute(), 30);
    assert_eq!(t.second(), 45);
}

#[test]
fn parse_cursor_value_time_invalid() {
    let result = parse_cursor_value(FieldKind::Time, "not-a-time");
    assert!(matches!(
        result,
        Err(ODataBuildError::Other("invalid time in cursor"))
    ));
}

#[test]
fn parse_cursor_value_decimal_success() {
    let result = parse_cursor_value(FieldKind::Decimal, "123.456").unwrap();
    let sea_orm::Value::Decimal(Some(d)) = result else {
        panic!("expected Decimal value");
    };
    assert_eq!(d.to_string(), "123.456");
}

#[test]
fn parse_cursor_value_decimal_preserves_scale() {
    let result = parse_cursor_value(FieldKind::Decimal, "123.400").unwrap();
    let sea_orm::Value::Decimal(Some(d)) = result else {
        panic!("expected Decimal value");
    };
    assert_eq!(d.to_string(), "123.400");
}

#[test]
fn parse_cursor_value_decimal_invalid() {
    let result = parse_cursor_value(FieldKind::Decimal, "not-a-decimal");
    assert!(matches!(
        result,
        Err(ODataBuildError::Other("invalid decimal in cursor"))
    ));
}

#[test]
fn build_cursor_for_model_success() {
    fn id_extractor(m: &Model) -> String {
        m.id.to_string()
    }
    fn name_extractor(m: &Model) -> String {
        m.name.clone()
    }

    let fmap = FieldMap::<Entity>::new()
        .insert_with_extractor("id", Column::Id, FieldKind::I64, id_extractor)
        .insert_with_extractor("name", Column::Name, FieldKind::String, name_extractor);

    let model = Model {
        id: 42,
        name: "test".to_owned(),
    };

    let order = ODataOrderBy(vec![
        OrderKey {
            field: "id".to_owned(),
            dir: SortDir::Asc,
        },
        OrderKey {
            field: "name".to_owned(),
            dir: SortDir::Desc,
        },
    ]);

    let cursor = build_cursor_for_model(
        &model,
        &order,
        &fmap,
        SortDir::Asc,
        Some("hash123".to_owned()),
        "fwd",
    )
    .unwrap();

    assert_eq!(cursor.k, vec!["42".to_owned(), "test".to_owned()]);
    assert_eq!(cursor.o, SortDir::Asc);
    assert_eq!(cursor.s, "+id,-name");
    assert_eq!(cursor.f, Some("hash123".to_owned()));
    assert_eq!(cursor.d, "fwd");
}

#[test]
fn build_cursor_for_model_missing_extractor() {
    let fmap = FieldMap::<Entity>::new().insert("id", Column::Id, FieldKind::I64);

    let model = Model {
        id: 42,
        name: "test".to_owned(),
    };

    let order = ODataOrderBy(vec![OrderKey {
        field: "name".to_owned(),
        dir: SortDir::Asc,
    }]);

    let result = build_cursor_for_model(&model, &order, &fmap, SortDir::Asc, None, "fwd");

    assert!(matches!(
        result,
        Err(ODataError::InvalidOrderByField(ref field)) if field == "name"
    ));
}

#[test]
fn build_cursor_for_model_backward_direction() {
    fn id_extractor(m: &Model) -> String {
        m.id.to_string()
    }

    let fmap = FieldMap::<Entity>::new().insert_with_extractor(
        "id",
        Column::Id,
        FieldKind::I64,
        id_extractor,
    );

    let model = Model {
        id: 99,
        name: "ignored".to_owned(),
    };

    let order = ODataOrderBy(vec![OrderKey {
        field: "id".to_owned(),
        dir: SortDir::Desc,
    }]);

    let cursor =
        build_cursor_for_model(&model, &order, &fmap, SortDir::Desc, None, "bwd").unwrap();

    assert_eq!(cursor.k, vec!["99".to_owned()]);
    assert_eq!(cursor.o, SortDir::Desc);
    assert_eq!(cursor.s, "-id");
    assert_eq!(cursor.f, None);
    assert_eq!(cursor.d, "bwd");
}
