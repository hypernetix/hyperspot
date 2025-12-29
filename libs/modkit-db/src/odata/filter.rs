//! Type-safe `OData` filter representation that operates on DTO-level field identifiers.
//!
//! This module provides:
//! - `FilterField` trait for defining filterable fields on DTOs
//! - `FilterOp` enum for filter operations (eq, ne, contains, etc.)
//! - `FilterNode<F>` AST for representing filters in a DB-agnostic way
//! - Parsing from `OData` filter strings to `FilterNode<F>`
//!
//! # Design Goals
//!
//! 1. **Type Safety**: Use generated enums instead of raw strings for field names
//! 2. **Separation of Concerns**: Keep DB infrastructure details (`SeaORM`, Column enums)
//!    out of API and domain layers
//! 3. **Flexibility**: Enable mapping from DTO-level filters to any backend in the infrastructure layer

use modkit_odata::ast as odata_ast;
use std::fmt;
use thiserror::Error;

use crate::odata::FieldKind;

/// Re-export `ODataValue` from `modkit_odata` for use in filters
pub use modkit_odata::ast::Value as ODataValue;

/// Trait representing a set of filterable fields for a DTO.
///
/// This trait is typically implemented by a derive macro on DTO types.
/// It provides a type-safe way to refer to filterable fields without using raw strings.
///
/// # Example
///
/// ```
/// use modkit_db::odata::{FilterField, FieldKind};
/// use modkit_db_macros::ODataFilterable;
///
/// // Define a DTO with filterable fields using the derive macro
/// #[derive(ODataFilterable)]
/// pub struct UserDto {
///     #[odata(filter(kind = "Uuid"))]
///     pub id: uuid::Uuid,
///     #[odata(filter(kind = "String"))]
///     pub email: String,
///     pub internal_field: String,  // not filterable
/// }
///
/// // The derive macro generates:
/// // - An enum `UserDtoFilterField` with variants for each filterable field
/// // - An implementation of `FilterField` trait for that enum
///
/// // Now you can use the generated type:
/// assert_eq!(UserDtoFilterField::from_name("email"), Some(UserDtoFilterField::Email));
/// assert_eq!(UserDtoFilterField::Email.name(), "email");
/// assert_eq!(UserDtoFilterField::Id.kind(), FieldKind::Uuid);
/// assert_eq!(UserDtoFilterField::FIELDS.len(), 2);
/// ```
pub trait FilterField: Copy + Eq + std::hash::Hash + fmt::Debug + 'static {
    /// All allowed fields for this DTO.
    const FIELDS: &'static [Self];

    /// API-visible name for this field (e.g., "email", "`created_at`").
    /// This is the name used in `OData` filter strings.
    fn name(&self) -> &'static str;

    /// Logical type of the field for value coercion and validation.
    fn kind(&self) -> FieldKind;

    /// Resolve a field by its API name, or None if not supported.
    fn from_name(name: &str) -> Option<Self> {
        Self::FIELDS
            .iter()
            .copied()
            .find(|f| f.name().eq_ignore_ascii_case(name))
    }
}

/// Filter operations supported in `OData` filters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterOp {
    /// Equality: field eq value
    Eq,
    /// Inequality: field ne value
    Ne,
    /// Greater than: field gt value
    Gt,
    /// Greater than or equal: field ge value
    Ge,
    /// Less than: field lt value
    Lt,
    /// Less than or equal: field le value
    Le,
    /// String contains: contains(field, 'substring')
    Contains,
    /// String starts with: startswith(field, 'prefix')
    StartsWith,
    /// String ends with: endswith(field, 'suffix')
    EndsWith,
    /// Logical AND
    And,
    /// Logical OR
    Or,
}

impl fmt::Display for FilterOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FilterOp::Eq => write!(f, "eq"),
            FilterOp::Ne => write!(f, "ne"),
            FilterOp::Gt => write!(f, "gt"),
            FilterOp::Ge => write!(f, "ge"),
            FilterOp::Lt => write!(f, "lt"),
            FilterOp::Le => write!(f, "le"),
            FilterOp::Contains => write!(f, "contains"),
            FilterOp::StartsWith => write!(f, "startswith"),
            FilterOp::EndsWith => write!(f, "endswith"),
            FilterOp::And => write!(f, "and"),
            FilterOp::Or => write!(f, "or"),
        }
    }
}

/// Type-safe filter AST node parameterized by a `FilterField` implementation.
///
/// This represents a filter expression in a database-agnostic way, using only
/// DTO-level field identifiers and logical operations.
///
/// # Type Parameters
///
/// - `F`: The `FilterField` implementation (typically a generated enum)
///
/// # Example
///
/// ```
/// use modkit_db::odata::filter::{FilterField, FilterNode, FilterOp, ODataValue, parse_odata_filter};
/// use modkit_db::odata::FieldKind;
/// use modkit_db_macros::ODataFilterable;
///
/// #[derive(ODataFilterable)]
/// pub struct UserDto {
///     #[odata(filter(kind = "String"))]
///     pub email: String,
/// }
///
/// // Parse from OData string (requires with-odata-params feature)
/// // let filter = parse_odata_filter::<UserDtoFilterField>("email eq 'test@example.com'")?;
///
/// // Or build manually using enum variants
/// use FilterNode::*;
/// let filter = Binary {
///     field: UserDtoFilterField::Email,
///     op: FilterOp::Eq,
///     value: ODataValue::String("test@example.com".to_string()),
/// };
///
/// // Or use the builder method
/// let filter2 = FilterNode::binary(
///     UserDtoFilterField::Email,
///     FilterOp::Eq,
///     ODataValue::String("test@example.com".to_string()),
/// );
/// ```
#[derive(Debug, Clone)]
pub enum FilterNode<F: FilterField> {
    /// Binary comparison: field op value
    Binary {
        field: F,
        op: FilterOp,
        value: ODataValue,
    },
    /// Composite expression: AND or OR of multiple filters
    Composite {
        op: FilterOp, // And or Or
        children: Vec<FilterNode<F>>,
    },
    /// Negation: NOT expression
    Not(Box<FilterNode<F>>),
}

impl<F: FilterField> FilterNode<F> {
    /// Create a simple binary comparison node
    pub fn binary(field: F, op: FilterOp, value: ODataValue) -> Self {
        FilterNode::Binary { field, op, value }
    }

    /// Create an AND composite node
    #[must_use]
    pub fn and(children: Vec<FilterNode<F>>) -> Self {
        FilterNode::Composite {
            op: FilterOp::And,
            children,
        }
    }

    /// Create an OR composite node
    #[must_use]
    pub fn or(children: Vec<FilterNode<F>>) -> Self {
        FilterNode::Composite {
            op: FilterOp::Or,
            children,
        }
    }

    /// Create a NOT node
    #[allow(clippy::should_implement_trait)]
    pub fn not(inner: FilterNode<F>) -> Self {
        FilterNode::Not(Box::new(inner))
    }
}

/// Errors that can occur during filter parsing or validation
#[derive(Debug, Error, Clone)]
pub enum FilterError {
    #[error("Unknown field: {0}")]
    UnknownField(String),

    #[error("Type mismatch for field {field}: expected {expected}, got {got}")]
    TypeMismatch {
        field: String,
        expected: FieldKind,
        got: &'static str,
    },

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("Invalid filter expression: {0}")]
    InvalidExpression(String),

    #[error("Field-to-field comparisons are not supported")]
    FieldToFieldComparison,

    #[error("Bare identifier in filter: {0}")]
    BareIdentifier(String),

    #[error("Bare literal in filter")]
    BareLiteral,
}

pub type FilterResult<T> = Result<T, FilterError>;

/// Parse an `OData` filter string into a `FilterNode`<F>.
///
/// This function takes a raw `OData` filter string (e.g., from a query parameter)
/// and converts it into a type-safe `FilterNode` using the provided `FilterField` implementation.
///
/// **Note**: This function requires the `with-odata-params` feature to be enabled
/// on the `modkit-odata` crate for actual parsing. If you're working with an
/// already-parsed AST (e.g., from `ODataQuery`), use `convert_expr_to_filter_node` directly.
///
/// # Type Parameters
///
/// - `F`: The `FilterField` implementation that defines available fields
///
/// # Arguments
///
/// - `raw`: The raw `OData` filter string (e.g., "email eq 'test@example.com'")
///
/// # Returns
///
/// A `FilterNode`<F> representing the parsed filter, or a `FilterError` if parsing fails.
///
/// # Example
///
/// ```ignore
/// let filter = parse_odata_filter::<UserDtoFilterField>(
///     "email eq 'test@example.com' and contains(display_name, 'John')"
/// )?;
/// ```
///
/// # Errors
/// Returns `FilterError` if the filter expression is invalid or parsing fails.
#[allow(unexpected_cfgs)]
pub fn parse_odata_filter<F: FilterField>(raw: &str) -> FilterResult<FilterNode<F>> {
    // Parse using odata-params (requires with-odata-params feature)
    #[cfg(feature = "with-odata-params")]
    {
        use odata_params;
        let ast = odata_params::parse_str(raw)
            .map_err(|e| FilterError::InvalidExpression(format!("{:?}", e)))?;
        // Convert from odata_params AST to modkit_odata AST
        let ast: odata_ast::Expr = ast.into();
        convert_expr_to_filter_node::<F>(&ast)
    }

    #[cfg(not(feature = "with-odata-params"))]
    {
        let _ = raw; // Suppress unused variable warning
        Err(FilterError::InvalidExpression(
            "OData filter parsing requires 'with-odata-params' feature".to_owned(),
        ))
    }
}

/// Convert `modkit_odata` AST expression to our `FilterNode`.
///
/// This function is useful when you already have a parsed AST (e.g., from `ODataQuery.filter`)
/// and want to convert it to a type-safe `FilterNode`.
///
/// # Example
///
/// ```rust
/// use modkit_db::odata::filter::{FilterNode, convert_expr_to_filter_node};
/// use modkit_db_macros::ODataFilterable;
/// use modkit_odata::{ast as odata_ast, ODataQuery};
///
/// // Define a DTO with filterable fields using the derive macro
/// #[derive(ODataFilterable)]
/// pub struct UserDto {
///     #[odata(filter(kind = "String"))]
///     pub email: String,
/// }
///
/// // Build an OData query with a filter expression
/// let expr = odata_ast::Expr::Compare(
///     Box::new(odata_ast::Expr::Identifier("email".to_owned())),
///     odata_ast::CompareOperator::Eq,
///     Box::new(odata_ast::Expr::Value(odata_ast::Value::String("test@example.com".to_owned()))),
/// );
/// let odata_query = ODataQuery::new().with_filter(expr);
///
/// // Convert from ODataQuery filter to type-safe FilterNode
/// let ast = odata_query.filter().unwrap();
/// let filter_node = convert_expr_to_filter_node::<UserDtoFilterField>(ast).unwrap();
/// assert!(matches!(filter_node, FilterNode::Binary { .. }));
///
/// ```
///
/// # Errors
/// Returns `FilterError` if the expression contains unknown fields or unsupported operations.
pub fn convert_expr_to_filter_node<F: FilterField>(
    expr: &odata_ast::Expr,
) -> FilterResult<FilterNode<F>> {
    use odata_ast::Expr as E;

    match expr {
        // Logical operators
        E::And(left, right) => {
            let left_node = convert_expr_to_filter_node::<F>(left)?;
            let right_node = convert_expr_to_filter_node::<F>(right)?;
            Ok(FilterNode::and(vec![left_node, right_node]))
        }
        E::Or(left, right) => {
            let left_node = convert_expr_to_filter_node::<F>(left)?;
            let right_node = convert_expr_to_filter_node::<F>(right)?;
            Ok(FilterNode::or(vec![left_node, right_node]))
        }
        E::Not(inner) => {
            let inner_node = convert_expr_to_filter_node::<F>(inner)?;
            Ok(FilterNode::not(inner_node))
        }

        // Binary comparisons
        E::Compare(left, op, right) => {
            // Extract field name and value
            let (field_name, value) = match (&**left, &**right) {
                (E::Identifier(name), E::Value(val)) => (name.as_str(), val.clone()),
                (E::Identifier(_), E::Identifier(_)) => {
                    return Err(FilterError::FieldToFieldComparison);
                }
                _ => {
                    return Err(FilterError::InvalidExpression(
                        "Comparison must be between field and value".to_owned(),
                    ));
                }
            };

            // Resolve field
            let field = F::from_name(field_name)
                .ok_or_else(|| FilterError::UnknownField(field_name.to_owned()))?;

            // Validate value type matches field kind
            validate_value_type(field, &value)?;

            // Convert operation
            let filter_op = match op {
                odata_ast::CompareOperator::Eq => FilterOp::Eq,
                odata_ast::CompareOperator::Ne => FilterOp::Ne,
                odata_ast::CompareOperator::Gt => FilterOp::Gt,
                odata_ast::CompareOperator::Ge => FilterOp::Ge,
                odata_ast::CompareOperator::Lt => FilterOp::Lt,
                odata_ast::CompareOperator::Le => FilterOp::Le,
            };

            Ok(FilterNode::binary(field, filter_op, value))
        }

        // Function calls (contains, startswith, endswith)
        E::Function(func_name, args) => {
            let name_lower = func_name.to_ascii_lowercase();
            match (name_lower.as_str(), args.as_slice()) {
                (
                    "contains",
                    [E::Identifier(field_name), E::Value(odata_ast::Value::String(s))],
                ) => {
                    let field = F::from_name(field_name)
                        .ok_or_else(|| FilterError::UnknownField(field_name.clone()))?;

                    // Ensure field is string type
                    if field.kind() != FieldKind::String {
                        return Err(FilterError::TypeMismatch {
                            field: field_name.clone(),
                            expected: FieldKind::String,
                            got: "non-string",
                        });
                    }

                    Ok(FilterNode::binary(
                        field,
                        FilterOp::Contains,
                        odata_ast::Value::String(s.clone()),
                    ))
                }
                (
                    "startswith",
                    [E::Identifier(field_name), E::Value(odata_ast::Value::String(s))],
                ) => {
                    let field = F::from_name(field_name)
                        .ok_or_else(|| FilterError::UnknownField(field_name.clone()))?;

                    if field.kind() != FieldKind::String {
                        return Err(FilterError::TypeMismatch {
                            field: field_name.clone(),
                            expected: FieldKind::String,
                            got: "non-string",
                        });
                    }

                    Ok(FilterNode::binary(
                        field,
                        FilterOp::StartsWith,
                        odata_ast::Value::String(s.clone()),
                    ))
                }
                (
                    "endswith",
                    [E::Identifier(field_name), E::Value(odata_ast::Value::String(s))],
                ) => {
                    let field = F::from_name(field_name)
                        .ok_or_else(|| FilterError::UnknownField(field_name.clone()))?;

                    if field.kind() != FieldKind::String {
                        return Err(FilterError::TypeMismatch {
                            field: field_name.clone(),
                            expected: FieldKind::String,
                            got: "non-string",
                        });
                    }

                    Ok(FilterNode::binary(
                        field,
                        FilterOp::EndsWith,
                        odata_ast::Value::String(s.clone()),
                    ))
                }
                _ => Err(FilterError::UnsupportedOperation(format!(
                    "Function '{func_name}'"
                ))),
            }
        }

        // IN operator
        E::In(_left, _list) => {
            // For now, we don't support IN in the simplified API
            // It can be added later if needed
            Err(FilterError::UnsupportedOperation(
                "IN operator not yet supported in typed filters".to_owned(),
            ))
        }

        // Invalid leaf expressions
        E::Identifier(name) => Err(FilterError::BareIdentifier(name.clone())),
        E::Value(_) => Err(FilterError::BareLiteral),
    }
}

/// Validate that a value matches the expected field kind
fn validate_value_type<F: FilterField>(field: F, value: &odata_ast::Value) -> FilterResult<()> {
    use odata_ast::Value as V;

    let got_type = match value {
        V::String(_) => "string",
        V::Number(_) => "number",
        V::Bool(_) => "bool",
        V::Uuid(_) => "uuid",
        V::DateTime(_) => "datetime",
        V::Date(_) => "date",
        V::Time(_) => "time",
        V::Null => "null",
    };

    let kind = field.kind();
    let matches = matches!(
        (kind, value),
        (FieldKind::String, V::String(_))
            | (
                FieldKind::I64 | FieldKind::F64 | FieldKind::Decimal,
                V::Number(_)
            )
            | (FieldKind::Bool, V::Bool(_))
            | (FieldKind::Uuid, V::Uuid(_))
            | (FieldKind::DateTimeUtc, V::DateTime(_))
            | (FieldKind::Date, V::Date(_))
            | (FieldKind::Time, V::Time(_))
    );

    if matches {
        Ok(())
    } else {
        Err(FilterError::TypeMismatch {
            field: field.name().to_owned(),
            expected: kind,
            got: got_type,
        })
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

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
}
