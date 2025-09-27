#[cfg(feature = "sea-orm")]
mod tests {
    use bigdecimal::BigDecimal;
    use sea_orm::entity::prelude::*;
    use std::str::FromStr;

    use modkit_db::odata::{expr_to_condition, FieldKind, FieldMap};
    use odata_core::ast::{CompareOperator, Expr, Value};

    // Simple test entity for compilation tests
    #[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "test_users")]
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

    fn setup_field_map() -> FieldMap<Entity> {
        FieldMap::<Entity>::new()
            .insert("id", Column::Id, FieldKind::Uuid)
            .insert("name", Column::Name, FieldKind::String)
            .insert("score", Column::Score, FieldKind::I64)
            .insert("email", Column::Email, FieldKind::String)
    }

    #[test]
    fn test_simple_equality() {
        let ast = Expr::Compare(
            Box::new(Expr::Identifier("score".to_string())),
            CompareOperator::Eq,
            Box::new(Expr::Value(Value::Number(
                BigDecimal::from_str("42").unwrap(),
            ))),
        );

        let fmap = setup_field_map();
        let condition = expr_to_condition::<Entity>(&ast, &fmap).unwrap();

        // Just verify the condition was created successfully
        // The actual SQL generation is handled by SeaORM internally
        assert!(!condition.is_empty());
    }

    #[test]
    fn test_and_expression() {
        let ast = Expr::And(
            Box::new(Expr::Compare(
                Box::new(Expr::Identifier("score".to_string())),
                CompareOperator::Gt,
                Box::new(Expr::Value(Value::Number(
                    BigDecimal::from_str("10").unwrap(),
                ))),
            )),
            Box::new(Expr::Function(
                "contains".to_string(),
                vec![
                    Expr::Identifier("email".to_string()),
                    Expr::Value(Value::String("@test.com".to_string())),
                ],
            )),
        );

        let fmap = setup_field_map();
        let condition = expr_to_condition::<Entity>(&ast, &fmap).unwrap();

        // Just verify the condition was created successfully
        assert!(!condition.is_empty());
    }

    #[test]
    fn test_in_expression() {
        let ast = Expr::In(
            Box::new(Expr::Identifier("score".to_string())),
            vec![
                Expr::Value(Value::Number(BigDecimal::from_str("1").unwrap())),
                Expr::Value(Value::Number(BigDecimal::from_str("2").unwrap())),
                Expr::Value(Value::Number(BigDecimal::from_str("3").unwrap())),
            ],
        );

        let fmap = setup_field_map();
        let condition = expr_to_condition::<Entity>(&ast, &fmap).unwrap();

        // Just verify the condition was created successfully
        assert!(!condition.is_empty());
    }

    #[test]
    fn test_null_comparison() {
        let ast = Expr::Compare(
            Box::new(Expr::Identifier("name".to_string())),
            CompareOperator::Eq,
            Box::new(Expr::Value(Value::Null)),
        );

        let fmap = setup_field_map();
        let condition = expr_to_condition::<Entity>(&ast, &fmap).unwrap();

        // Just verify the condition was created successfully
        assert!(!condition.is_empty());
    }

    #[test]
    fn test_unknown_field_error() {
        let ast = Expr::Compare(
            Box::new(Expr::Identifier("unknown_field".to_string())),
            CompareOperator::Eq,
            Box::new(Expr::Value(Value::String("test".to_string()))),
        );

        let fmap = setup_field_map();
        let result = expr_to_condition::<Entity>(&ast, &fmap);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("unknown field"));
    }
}
