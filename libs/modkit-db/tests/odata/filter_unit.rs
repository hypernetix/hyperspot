//! Unit tests for OData filter conversion (FilterNode-based)

use modkit_db::odata::{
    convert_expr_to_filter_node, FieldKind, FilterError, FilterField, FilterNode, FilterOp,
};
use modkit_odata::ast as odata_ast;

// Test FilterField implementation
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
enum TestField {
    Email,
    Age,
    IsActive,
}

impl FilterField for TestField {
    const FIELDS: &'static [Self] = &[TestField::Email, TestField::Age, TestField::IsActive];

    fn name(&self) -> &'static str {
        match self {
            TestField::Email => "email",
            TestField::Age => "age",
            TestField::IsActive => "is_active",
        }
    }

    fn kind(&self) -> FieldKind {
        match self {
            TestField::Email => FieldKind::String,
            TestField::Age => FieldKind::I64,
            TestField::IsActive => FieldKind::Bool,
        }
    }
}

#[test]
fn test_filter_field_from_name() {
    assert_eq!(TestField::from_name("email"), Some(TestField::Email));
    assert_eq!(TestField::from_name("age"), Some(TestField::Age));
    assert_eq!(TestField::from_name("is_active"), Some(TestField::IsActive));
    assert_eq!(TestField::from_name("unknown"), None);
}

#[test]
fn test_filter_field_case_insensitive() {
    assert_eq!(TestField::from_name("EMAIL"), Some(TestField::Email));
    assert_eq!(TestField::from_name("Age"), Some(TestField::Age));
    assert_eq!(TestField::from_name("IS_ACTIVE"), Some(TestField::IsActive));
}

#[test]
fn test_convert_simple_eq_filter() {
    // Create an AST manually (simulating what would come from parsed OData)
    let ast = odata_ast::Expr::Compare(
        Box::new(odata_ast::Expr::Identifier("email".to_owned())),
        odata_ast::CompareOperator::Eq,
        Box::new(odata_ast::Expr::Value(odata_ast::Value::String(
            "test@example.com".to_owned(),
        ))),
    );

    let result = convert_expr_to_filter_node::<TestField>(&ast);
    assert!(result.is_ok());

    if let Ok(FilterNode::Binary { field, op, value }) = result {
        assert_eq!(field, TestField::Email);
        assert_eq!(op, FilterOp::Eq);
        assert!(matches!(value, odata_ast::Value::String(_)));
    } else {
        panic!("Expected Binary node");
    }
}

#[test]
fn test_convert_unknown_field() {
    let ast = odata_ast::Expr::Compare(
        Box::new(odata_ast::Expr::Identifier("unknown_field".to_owned())),
        odata_ast::CompareOperator::Eq,
        Box::new(odata_ast::Expr::Value(odata_ast::Value::String(
            "value".to_owned(),
        ))),
    );

    let result = convert_expr_to_filter_node::<TestField>(&ast);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), FilterError::UnknownField(_)));
}

#[test]
fn test_validate_type_mismatch() {
    // Try to use a string value for an integer field - should fail validation
    let ast = odata_ast::Expr::Compare(
        Box::new(odata_ast::Expr::Identifier("age".to_owned())),
        odata_ast::CompareOperator::Eq,
        Box::new(odata_ast::Expr::Value(odata_ast::Value::String(
            "not_a_number".to_owned(),
        ))),
    );

    let result = convert_expr_to_filter_node::<TestField>(&ast);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        FilterError::TypeMismatch { .. }
    ));
}

#[test]
fn test_logical_and_combination() {
    // (email eq 'test@example.com') and (is_active eq true)
    let left = odata_ast::Expr::Compare(
        Box::new(odata_ast::Expr::Identifier("email".to_owned())),
        odata_ast::CompareOperator::Eq,
        Box::new(odata_ast::Expr::Value(odata_ast::Value::String(
            "test@example.com".to_owned(),
        ))),
    );
    let right = odata_ast::Expr::Compare(
        Box::new(odata_ast::Expr::Identifier("is_active".to_owned())),
        odata_ast::CompareOperator::Eq,
        Box::new(odata_ast::Expr::Value(odata_ast::Value::Bool(true))),
    );
    let ast = odata_ast::Expr::And(Box::new(left), Box::new(right));

    let result = convert_expr_to_filter_node::<TestField>(&ast);
    assert!(result.is_ok());

    if let Ok(FilterNode::Composite { op, children }) = result {
        assert_eq!(op, FilterOp::And);
        assert_eq!(children.len(), 2);
    } else {
        panic!("Expected Composite And node");
    }
}

#[test]
fn test_logical_or_combination() {
    // (age gt 30) or (is_active eq false)
    let left = odata_ast::Expr::Compare(
        Box::new(odata_ast::Expr::Identifier("age".to_owned())),
        odata_ast::CompareOperator::Gt,
        Box::new(odata_ast::Expr::Value(odata_ast::Value::Number(30.into()))),
    );
    let right = odata_ast::Expr::Compare(
        Box::new(odata_ast::Expr::Identifier("is_active".to_owned())),
        odata_ast::CompareOperator::Eq,
        Box::new(odata_ast::Expr::Value(odata_ast::Value::Bool(false))),
    );
    let ast = odata_ast::Expr::Or(Box::new(left), Box::new(right));

    let result = convert_expr_to_filter_node::<TestField>(&ast);
    assert!(result.is_ok());

    if let Ok(FilterNode::Composite { op, children }) = result {
        assert_eq!(op, FilterOp::Or);
        assert_eq!(children.len(), 2);
    } else {
        panic!("Expected Composite Or node");
    }
}

#[test]
fn test_logical_not() {
    // not (age eq 25)
    let inner = odata_ast::Expr::Compare(
        Box::new(odata_ast::Expr::Identifier("age".to_owned())),
        odata_ast::CompareOperator::Eq,
        Box::new(odata_ast::Expr::Value(odata_ast::Value::Number(25.into()))),
    );
    let ast = odata_ast::Expr::Not(Box::new(inner));

    let result = convert_expr_to_filter_node::<TestField>(&ast);
    assert!(result.is_ok());

    if let Ok(FilterNode::Not(inner_node)) = result {
        assert!(matches!(*inner_node, FilterNode::Binary { .. }));
    } else {
        panic!("Expected Not node");
    }
}

#[test]
fn test_logical_not_composite() {
    // not ((email eq 'test') and (age gt 20))
    let email_cond = odata_ast::Expr::Compare(
        Box::new(odata_ast::Expr::Identifier("email".to_owned())),
        odata_ast::CompareOperator::Eq,
        Box::new(odata_ast::Expr::Value(odata_ast::Value::String(
            "test".to_owned(),
        ))),
    );
    let age_cond = odata_ast::Expr::Compare(
        Box::new(odata_ast::Expr::Identifier("age".to_owned())),
        odata_ast::CompareOperator::Gt,
        Box::new(odata_ast::Expr::Value(odata_ast::Value::Number(20.into()))),
    );
    let and_expr = odata_ast::Expr::And(Box::new(email_cond), Box::new(age_cond));
    let ast = odata_ast::Expr::Not(Box::new(and_expr));

    let result = convert_expr_to_filter_node::<TestField>(&ast);
    assert!(result.is_ok());

    if let Ok(FilterNode::Not(inner_node)) = result {
        assert!(matches!(*inner_node, FilterNode::Composite { .. }));
    } else {
        panic!("Expected Not with Composite inner node");
    }
}

#[test]
fn test_contains_function() {
    // contains(email, 'test')
    let ast = odata_ast::Expr::Function(
        "contains".to_owned(),
        vec![
            odata_ast::Expr::Identifier("email".to_owned()),
            odata_ast::Expr::Value(odata_ast::Value::String("test".to_owned())),
        ],
    );

    let result = convert_expr_to_filter_node::<TestField>(&ast);
    assert!(result.is_ok());

    if let Ok(FilterNode::Binary { field, op, value }) = result {
        assert_eq!(field, TestField::Email);
        assert_eq!(op, FilterOp::Contains);
        assert!(matches!(value, odata_ast::Value::String(_)));
    } else {
        panic!("Expected Binary node with Contains operation");
    }
}

#[test]
fn test_startswith_function() {
    // startswith(email, 'test')
    let ast = odata_ast::Expr::Function(
        "startswith".to_owned(),
        vec![
            odata_ast::Expr::Identifier("email".to_owned()),
            odata_ast::Expr::Value(odata_ast::Value::String("test".to_owned())),
        ],
    );

    let result = convert_expr_to_filter_node::<TestField>(&ast);
    assert!(result.is_ok());

    if let Ok(FilterNode::Binary { field, op, .. }) = result {
        assert_eq!(field, TestField::Email);
        assert_eq!(op, FilterOp::StartsWith);
    } else {
        panic!("Expected Binary node with StartsWith operation");
    }
}

#[test]
fn test_endswith_function() {
    // endswith(email, '.com')
    let ast = odata_ast::Expr::Function(
        "endswith".to_owned(),
        vec![
            odata_ast::Expr::Identifier("email".to_owned()),
            odata_ast::Expr::Value(odata_ast::Value::String(".com".to_owned())),
        ],
    );

    let result = convert_expr_to_filter_node::<TestField>(&ast);
    assert!(result.is_ok());

    if let Ok(FilterNode::Binary { field, op, .. }) = result {
        assert_eq!(field, TestField::Email);
        assert_eq!(op, FilterOp::EndsWith);
    } else {
        panic!("Expected Binary node with EndsWith operation");
    }
}

#[test]
fn test_contains_on_non_string_field_fails() {
    // contains(age, 'test') - should fail because age is I64, not String
    let ast = odata_ast::Expr::Function(
        "contains".to_owned(),
        vec![
            odata_ast::Expr::Identifier("age".to_owned()),
            odata_ast::Expr::Value(odata_ast::Value::String("test".to_owned())),
        ],
    );

    let result = convert_expr_to_filter_node::<TestField>(&ast);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        FilterError::TypeMismatch { .. }
    ));
}

#[test]
fn test_bare_identifier_error() {
    // Just "email" by itself is not valid
    let ast = odata_ast::Expr::Identifier("email".to_owned());

    let result = convert_expr_to_filter_node::<TestField>(&ast);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        FilterError::BareIdentifier(_)
    ));
}

#[test]
fn test_bare_literal_error() {
    // Just a string literal by itself is not valid
    let ast = odata_ast::Expr::Value(odata_ast::Value::String("test".to_owned()));

    let result = convert_expr_to_filter_node::<TestField>(&ast);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), FilterError::BareLiteral));
}

#[test]
fn test_unsupported_function() {
    // substring() is not supported
    let ast = odata_ast::Expr::Function(
        "substring".to_owned(),
        vec![
            odata_ast::Expr::Identifier("email".to_owned()),
            odata_ast::Expr::Value(odata_ast::Value::Number(1.into())),
        ],
    );

    let result = convert_expr_to_filter_node::<TestField>(&ast);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        FilterError::UnsupportedOperation(_)
    ));
}

// Test with a Decimal field
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
enum TestFieldWithDecimal {
    Name,
    Price,
}

impl FilterField for TestFieldWithDecimal {
    const FIELDS: &'static [Self] = &[TestFieldWithDecimal::Name, TestFieldWithDecimal::Price];

    fn name(&self) -> &'static str {
        match self {
            TestFieldWithDecimal::Name => "name",
            TestFieldWithDecimal::Price => "price",
        }
    }

    fn kind(&self) -> FieldKind {
        match self {
            TestFieldWithDecimal::Name => FieldKind::String,
            TestFieldWithDecimal::Price => FieldKind::Decimal,
        }
    }
}

#[test]
fn test_decimal_field_validation() {
    use bigdecimal::BigDecimal;
    use std::str::FromStr;

    // price eq 19.99
    let ast = odata_ast::Expr::Compare(
        Box::new(odata_ast::Expr::Identifier("price".to_owned())),
        odata_ast::CompareOperator::Eq,
        Box::new(odata_ast::Expr::Value(odata_ast::Value::Number(
            BigDecimal::from_str("19.99").unwrap(),
        ))),
    );

    let result = convert_expr_to_filter_node::<TestFieldWithDecimal>(&ast);
    assert!(result.is_ok());

    if let Ok(FilterNode::Binary { field, .. }) = result {
        assert_eq!(field, TestFieldWithDecimal::Price);
    } else {
        panic!("Expected Binary node");
    }
}

#[test]
fn test_decimal_field_wrong_type() {
    // price eq true - should fail
    let ast = odata_ast::Expr::Compare(
        Box::new(odata_ast::Expr::Identifier("price".to_owned())),
        odata_ast::CompareOperator::Eq,
        Box::new(odata_ast::Expr::Value(odata_ast::Value::Bool(true))),
    );

    let result = convert_expr_to_filter_node::<TestFieldWithDecimal>(&ast);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        FilterError::TypeMismatch { .. }
    ));
}
