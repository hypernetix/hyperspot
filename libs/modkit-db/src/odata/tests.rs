#[cfg(test)]
mod tests {
    use crate::odata::*;
    use modkit_odata::{ast::*, CursorV1, ODataOrderBy, OrderKey, SortDir};

    // Mock entity for testing
    #[derive(Debug, Clone, PartialEq, sea_orm::EntityTrait)]
    pub struct TestEntity;

    #[derive(Debug, Clone, PartialEq, sea_orm::EnumIter, sea_orm::DeriveColumn)]
    pub enum TestColumn {
        Id,
        Name,
        CreatedAt,
        Score,
    }

    impl sea_orm::ColumnTrait for TestColumn {
        type EntityName = TestEntity;

        fn def(&self) -> sea_orm::ColumnDef {
            match self {
                Self::Id => sea_orm::ColumnType::Uuid.def(),
                Self::Name => sea_orm::ColumnType::String(Some(255)).def(),
                Self::CreatedAt => sea_orm::ColumnType::TimestampWithTimeZone.def(),
                Self::Score => sea_orm::ColumnType::Double.def(),
            }
        }
    }

    impl sea_orm::EntityName for TestEntity {
        fn table_name(&self) -> &str {
            "test"
        }
    }

    impl sea_orm::PrimaryKeyTrait for TestEntity {
        type ValueType = uuid::Uuid;

        fn auto_increment() -> bool {
            false
        }
    }

    impl sea_orm::ModelTrait for TestEntity {
        type Entity = TestEntity;

        fn get(&self, _c: <Self::Entity as sea_orm::EntityTrait>::Column) -> sea_orm::Value {
            unreachable!()
        }

        fn set(&mut self, _c: <Self::Entity as sea_orm::EntityTrait>::Column, _v: sea_orm::Value) {
            unreachable!()
        }
    }

    impl sea_orm::ActiveModelTrait for TestEntity {
        type Entity = TestEntity;

        fn take(
            &mut self,
            _c: <Self::Entity as sea_orm::EntityTrait>::Column,
        ) -> sea_orm::ActiveValue<sea_orm::Value> {
            unreachable!()
        }

        fn get(
            &self,
            _c: <Self::Entity as sea_orm::EntityTrait>::Column,
        ) -> &sea_orm::ActiveValue<sea_orm::Value> {
            unreachable!()
        }

        fn set(
            &mut self,
            _c: <Self::Entity as sea_orm::EntityTrait>::Column,
            _v: sea_orm::ActiveValue<sea_orm::Value>,
        ) {
            unreachable!()
        }

        fn not_set(&mut self, _c: <Self::Entity as sea_orm::EntityTrait>::Column) {
            unreachable!()
        }

        fn is_not_set(&self, _c: <Self::Entity as sea_orm::EntityTrait>::Column) -> bool {
            unreachable!()
        }
    }

    type TestFieldMap = FieldMap<TestEntity>;

    fn test_field_map() -> TestFieldMap {
        FieldMap::<TestEntity>::new()
            .insert("id", TestColumn::Id, FieldKind::Uuid)
            .insert("name", TestColumn::Name, FieldKind::String)
            .insert("created_at", TestColumn::CreatedAt, FieldKind::DateTimeUtc)
            .insert("score", TestColumn::Score, FieldKind::F64)
    }

    #[test]
    fn test_encode_cursor_value_string() {
        let value = sea_orm::Value::String(Some(Box::new("test".to_string())));
        let result = encode_cursor_value(FieldKind::String, &value).unwrap();
        assert_eq!(result, "test");
    }

    #[test]
    fn test_encode_cursor_value_i64() {
        let value = sea_orm::Value::BigInt(Some(123));
        let result = encode_cursor_value(FieldKind::I64, &value).unwrap();
        assert_eq!(result, "123");
    }

    #[test]
    fn test_encode_cursor_value_f64() {
        let value = sea_orm::Value::Double(Some(123.456));
        let result = encode_cursor_value(FieldKind::F64, &value).unwrap();
        assert_eq!(result, "123.456");
    }

    #[test]
    fn test_encode_cursor_value_bool() {
        let value = sea_orm::Value::Bool(Some(true));
        let result = encode_cursor_value(FieldKind::Bool, &value).unwrap();
        assert_eq!(result, "true");
    }

    #[test]
    fn test_encode_cursor_value_uuid() {
        let uuid = uuid::Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap();
        let value = sea_orm::Value::Uuid(Some(Box::new(uuid)));
        let result = encode_cursor_value(FieldKind::Uuid, &value).unwrap();
        assert_eq!(result, "123e4567-e89b-12d3-a456-426614174000");
    }

    #[test]
    fn test_parse_cursor_value_string() {
        let result = parse_cursor_value(FieldKind::String, "test").unwrap();
        assert!(matches!(result, sea_orm::Value::String(Some(_))));
    }

    #[test]
    fn test_parse_cursor_value_i64() {
        let result = parse_cursor_value(FieldKind::I64, "123").unwrap();
        assert!(matches!(result, sea_orm::Value::BigInt(Some(123))));
    }

    #[test]
    fn test_parse_cursor_value_f64() {
        let result = parse_cursor_value(FieldKind::F64, "123.456").unwrap();
        assert!(
            matches!(result, sea_orm::Value::Double(Some(v)) if (v - 123.456).abs() < f64::EPSILON)
        );
    }

    #[test]
    fn test_parse_cursor_value_bool() {
        let result = parse_cursor_value(FieldKind::Bool, "true").unwrap();
        assert!(matches!(result, sea_orm::Value::Bool(Some(true))));
    }

    #[test]
    fn test_parse_cursor_value_uuid() {
        let result =
            parse_cursor_value(FieldKind::Uuid, "123e4567-e89b-12d3-a456-426614174000").unwrap();
        assert!(matches!(result, sea_orm::Value::Uuid(Some(_))));
    }

    #[test]
    fn test_parse_cursor_value_invalid() {
        let result = parse_cursor_value(FieldKind::I64, "not_a_number");
        assert!(result.is_err());
    }

    #[test]
    fn test_ensure_tiebreaker() {
        let order = ODataOrderBy(vec![OrderKey {
            field: "name".to_string(),
            dir: SortDir::Asc,
        }]);

        let result = ensure_tiebreaker(order, "id", SortDir::Desc);
        assert_eq!(result.0.len(), 2);
        assert_eq!(result.0[1].field, "id");
        assert_eq!(result.0[1].dir, SortDir::Desc);
    }

    #[test]
    fn test_build_cursor_predicate_single_field_asc() {
        let cursor = CursorV1 {
            k: vec!["test_value".to_string()],
            o: SortDir::Asc,
            s: "+name".to_string(),
            f: None,
            d: "fwd".to_string(),
        };

        let order = ODataOrderBy(vec![OrderKey {
            field: "name".to_string(),
            dir: SortDir::Asc,
        }]);

        let fmap = test_field_map();
        let result = build_cursor_predicate(&cursor, &order, &fmap);
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_cursor_predicate_single_field_desc() {
        let cursor = CursorV1 {
            k: vec!["test_value".to_string()],
            o: SortDir::Desc,
            s: "-name".to_string(),
            f: None,
            d: "fwd".to_string(),
        };

        let order = ODataOrderBy(vec![OrderKey {
            field: "name".to_string(),
            dir: SortDir::Desc,
        }]);

        let fmap = test_field_map();
        let result = build_cursor_predicate(&cursor, &order, &fmap);
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_cursor_predicate_multiple_fields() {
        let cursor = CursorV1 {
            k: vec!["2023-11-14T12:00:00Z".to_string(), "test_id".to_string()],
            o: SortDir::Desc,
            s: "-created_at,-id".to_string(),
            f: None,
            d: "fwd".to_string(),
        };

        let order = ODataOrderBy(vec![
            OrderKey {
                field: "created_at".to_string(),
                dir: SortDir::Desc,
            },
            OrderKey {
                field: "id".to_string(),
                dir: SortDir::Desc,
            },
        ]);

        let fmap = test_field_map();
        let result = build_cursor_predicate(&cursor, &order, &fmap);
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_cursor_predicate_key_count_mismatch() {
        let cursor = CursorV1 {
            k: vec!["value1".to_string()],
            o: SortDir::Asc,
            s: "+field1".to_string(),
            f: None,
            d: "fwd".to_string(),
        };

        let order = ODataOrderBy(vec![
            OrderKey {
                field: "field1".to_string(),
                dir: SortDir::Asc,
            },
            OrderKey {
                field: "field2".to_string(),
                dir: SortDir::Asc,
            },
        ]);

        let fmap = test_field_map();
        let result = build_cursor_predicate(&cursor, &order, &fmap);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cursor keys count mismatch"));
    }

    #[test]
    fn test_build_cursor_predicate_unknown_field() {
        let cursor = CursorV1 {
            k: vec!["value".to_string()],
            o: SortDir::Asc,
            s: "+unknown_field".to_string(),
            f: None,
            d: "fwd".to_string(),
        };

        let order = ODataOrderBy(vec![OrderKey {
            field: "unknown_field".to_string(),
            dir: SortDir::Asc,
        }]);

        let fmap = test_field_map();
        let result = build_cursor_predicate(&cursor, &order, &fmap);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ODataBuildError::UnknownField(_)
        ));
    }

    #[test]
    fn test_expr_to_condition_compare() {
        let expr = Expr::Compare(
            Box::new(Expr::Identifier("name".to_string())),
            CompareOperator::Eq,
            Box::new(Expr::Value(Value::String("test".to_string()))),
        );

        let fmap = test_field_map();
        let result = expr_to_condition(&expr, &fmap);
        assert!(result.is_ok());
    }

    #[test]
    fn test_expr_to_condition_and() {
        let expr = Expr::And(
            Box::new(Expr::Compare(
                Box::new(Expr::Identifier("name".to_string())),
                CompareOperator::Eq,
                Box::new(Expr::Value(Value::String("test".to_string()))),
            )),
            Box::new(Expr::Compare(
                Box::new(Expr::Identifier("score".to_string())),
                CompareOperator::Gt,
                Box::new(Expr::Value(Value::Number(bigdecimal::BigDecimal::from(10)))),
            )),
        );

        let fmap = test_field_map();
        let result = expr_to_condition(&expr, &fmap);
        assert!(result.is_ok());
    }

    #[test]
    fn test_expr_to_condition_unknown_field() {
        let expr = Expr::Compare(
            Box::new(Expr::Identifier("unknown_field".to_string())),
            CompareOperator::Eq,
            Box::new(Expr::Value(Value::String("test".to_string()))),
        );

        let fmap = test_field_map();
        let result = expr_to_condition(&expr, &fmap);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ODataBuildError::UnknownField(_)
        ));
    }

    #[test]
    fn test_expr_to_condition_function_contains() {
        let expr = Expr::Function(
            "contains".to_string(),
            vec![
                Expr::Identifier("name".to_string()),
                Expr::Value(Value::String("test".to_string())),
            ],
        );

        let fmap = test_field_map();
        let result = expr_to_condition(&expr, &fmap);
        assert!(result.is_ok());
    }

    #[test]
    fn test_expr_to_condition_in() {
        let expr = Expr::In(
            Box::new(Expr::Identifier("name".to_string())),
            vec![
                Expr::Value(Value::String("test1".to_string())),
                Expr::Value(Value::String("test2".to_string())),
            ],
        );

        let fmap = test_field_map();
        let result = expr_to_condition(&expr, &fmap);
        assert!(result.is_ok());
    }
}
