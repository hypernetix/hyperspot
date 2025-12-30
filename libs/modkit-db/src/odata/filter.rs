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
        got: String,
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
                            got: "non-string".to_owned(),
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
                            got: "non-string".to_owned(),
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
                            got: "non-string".to_owned(),
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
            got: value.to_string(),
        })
    }
}
