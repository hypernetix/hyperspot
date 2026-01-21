//! Unit tests for SeaORM filter mapping and cursor encoding/decoding

use modkit_db::odata::{encode_cursor_value, parse_cursor_value};
use modkit_odata::filter::{FieldKind, ODataValue};

#[test]
fn test_escape_like() {
    use modkit_db::odata::sea_orm_filter::escape_like;
    assert_eq!(escape_like("test"), "test");
    assert_eq!(escape_like("test%"), "test\\%");
    assert_eq!(escape_like("test_name"), "test\\_name");
    assert_eq!(escape_like("test\\value"), "test\\\\value");
}

#[test]
fn test_odata_value_to_sea_value_string() {
    use modkit_db::odata::sea_orm_filter::odata_value_to_sea_value;
    let value = ODataValue::String("test".to_owned());
    let result = odata_value_to_sea_value(&value);
    assert!(result.is_ok());
}

#[test]
fn test_odata_value_to_sea_value_null() {
    use modkit_db::odata::sea_orm_filter::odata_value_to_sea_value;
    let value = ODataValue::Null;
    let result = odata_value_to_sea_value(&value);
    assert!(result.is_err());
}

#[test]
fn test_encode_decode_cursor_string() {
    use sea_orm::Value as V;
    let val = V::String(Some(Box::new("test".to_owned())));
    let encoded = encode_cursor_value(&val, FieldKind::String).unwrap();
    assert_eq!(encoded, "test");

    let decoded = parse_cursor_value(FieldKind::String, &encoded).unwrap();
    assert!(matches!(decoded, V::String(Some(_))));
}

#[test]
fn test_encode_decode_cursor_i64() {
    use sea_orm::Value as V;
    let val = V::BigInt(Some(42));
    let encoded = encode_cursor_value(&val, FieldKind::I64).unwrap();
    assert_eq!(encoded, "42");

    let decoded = parse_cursor_value(FieldKind::I64, &encoded).unwrap();
    assert!(matches!(decoded, V::BigInt(Some(42))));
}

#[test]
fn test_encode_decode_cursor_f64() {
    use sea_orm::Value as V;
    let val = V::Double(Some(3.5));
    let encoded = encode_cursor_value(&val, FieldKind::F64).unwrap();

    let decoded = parse_cursor_value(FieldKind::F64, &encoded).unwrap();
    if let V::Double(Some(f)) = decoded {
        assert!((f - 3.5).abs() < 0.001);
    } else {
        panic!("Expected Double value");
    }
}

#[test]
fn test_encode_decode_cursor_bool() {
    use sea_orm::Value as V;
    let val = V::Bool(Some(true));
    let encoded = encode_cursor_value(&val, FieldKind::Bool).unwrap();
    assert_eq!(encoded, "true");

    let decoded = parse_cursor_value(FieldKind::Bool, &encoded).unwrap();
    assert!(matches!(decoded, V::Bool(Some(true))));
}

#[test]
fn test_encode_decode_cursor_uuid() {
    use sea_orm::Value as V;
    let id = uuid::Uuid::new_v4();
    let val = V::Uuid(Some(Box::new(id)));
    let encoded = encode_cursor_value(&val, FieldKind::Uuid).unwrap();

    let decoded = parse_cursor_value(FieldKind::Uuid, &encoded).unwrap();
    if let V::Uuid(Some(decoded_id)) = decoded {
        assert_eq!(*decoded_id, id);
    } else {
        panic!("Expected UUID value");
    }
}

#[test]
fn test_encode_decode_cursor_datetime() {
    use chrono::{TimeZone, Utc};
    use sea_orm::Value as V;

    let dt = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
    let val = V::ChronoDateTimeUtc(Some(Box::new(dt)));
    let encoded = encode_cursor_value(&val, FieldKind::DateTimeUtc).unwrap();
    assert!(encoded.ends_with('Z'));
    assert!(encoded.contains(".000000000Z"));

    let decoded = parse_cursor_value(FieldKind::DateTimeUtc, &encoded).unwrap();
    match decoded {
        V::ChronoDateTimeUtc(Some(decoded_dt)) => {
            assert_eq!(*decoded_dt, dt);
        }
        V::TimeDateTimeWithTimeZone(Some(decoded_dt)) => {
            assert_eq!(decoded_dt.unix_timestamp(), dt.timestamp());
            assert_eq!(decoded_dt.nanosecond(), dt.timestamp_subsec_nanos());
        }
        _ => panic!("Expected DateTime value"),
    }
}

#[test]
fn test_encode_decode_cursor_date() {
    use chrono::NaiveDate;
    use sea_orm::Value as V;

    let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
    let val = V::ChronoDate(Some(Box::new(date)));
    let encoded = encode_cursor_value(&val, FieldKind::Date).unwrap();

    let decoded = parse_cursor_value(FieldKind::Date, &encoded).unwrap();
    if let V::ChronoDate(Some(decoded_date)) = decoded {
        assert_eq!(*decoded_date, date);
    } else {
        panic!("Expected Date value");
    }
}

#[test]
fn test_encode_decode_cursor_time() {
    use chrono::NaiveTime;
    use sea_orm::Value as V;

    let time = NaiveTime::from_hms_opt(10, 30, 45).unwrap();
    let val = V::ChronoTime(Some(Box::new(time)));
    let encoded = encode_cursor_value(&val, FieldKind::Time).unwrap();

    let decoded = parse_cursor_value(FieldKind::Time, &encoded).unwrap();
    if let V::ChronoTime(Some(decoded_time)) = decoded {
        assert_eq!(*decoded_time, time);
    } else {
        panic!("Expected Time value");
    }
}

#[test]
fn test_encode_decode_cursor_decimal() {
    use rust_decimal::Decimal;
    use sea_orm::Value as V;
    use std::str::FromStr;

    let dec = Decimal::from_str("19.99").unwrap();
    let val = V::Decimal(Some(Box::new(dec)));
    let encoded = encode_cursor_value(&val, FieldKind::Decimal).unwrap();

    let decoded = parse_cursor_value(FieldKind::Decimal, &encoded).unwrap();
    if let V::Decimal(Some(decoded_dec)) = decoded {
        assert_eq!(*decoded_dec, dec);
    } else {
        panic!("Expected Decimal value");
    }
}

#[test]
fn test_parse_cursor_invalid_i64() {
    let result = parse_cursor_value(FieldKind::I64, "not-a-number");
    assert!(result.is_err());
}

#[test]
fn test_parse_cursor_invalid_uuid() {
    let result = parse_cursor_value(FieldKind::Uuid, "not-a-uuid");
    assert!(result.is_err());
}

#[test]
fn test_parse_cursor_invalid_bool() {
    let result = parse_cursor_value(FieldKind::Bool, "maybe");
    assert!(result.is_err());
}
