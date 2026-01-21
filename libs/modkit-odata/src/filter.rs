use std::fmt;

use thiserror::Error;

use crate::ast as odata_ast;

pub use crate::ast::Value as ODataValue;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldKind {
    String,
    I64,
    F64,
    Bool,
    Uuid,
    DateTimeUtc,
    Date,
    Time,
    Decimal,
}

impl fmt::Display for FieldKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FieldKind::String => write!(f, "String"),
            FieldKind::I64 => write!(f, "I64"),
            FieldKind::F64 => write!(f, "F64"),
            FieldKind::Bool => write!(f, "Bool"),
            FieldKind::Uuid => write!(f, "Uuid"),
            FieldKind::DateTimeUtc => write!(f, "DateTimeUtc"),
            FieldKind::Date => write!(f, "Date"),
            FieldKind::Time => write!(f, "Time"),
            FieldKind::Decimal => write!(f, "Decimal"),
        }
    }
}

pub trait FilterField: Copy + Eq + std::hash::Hash + fmt::Debug + 'static {
    const FIELDS: &'static [Self];

    fn name(&self) -> &'static str;

    fn kind(&self) -> FieldKind;

    fn from_name(name: &str) -> Option<Self> {
        Self::FIELDS
            .iter()
            .copied()
            .find(|f| f.name().eq_ignore_ascii_case(name))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterOp {
    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,
    Contains,
    StartsWith,
    EndsWith,
    And,
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

#[derive(Debug, Clone)]
pub enum FilterNode<F: FilterField> {
    Binary {
        field: F,
        op: FilterOp,
        value: ODataValue,
    },
    Composite {
        op: FilterOp,
        children: Vec<FilterNode<F>>,
    },
    Not(Box<FilterNode<F>>),
}

impl<F: FilterField> FilterNode<F> {
    pub fn binary(field: F, op: FilterOp, value: ODataValue) -> Self {
        FilterNode::Binary { field, op, value }
    }

    #[must_use]
    pub fn and(children: Vec<FilterNode<F>>) -> Self {
        FilterNode::Composite {
            op: FilterOp::And,
            children,
        }
    }

    #[must_use]
    pub fn or(children: Vec<FilterNode<F>>) -> Self {
        FilterNode::Composite {
            op: FilterOp::Or,
            children,
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn not(inner: FilterNode<F>) -> Self {
        FilterNode::Not(Box::new(inner))
    }
}

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

#[allow(unexpected_cfgs)]
/// Parse an `OData` filter string into a typed `FilterNode`.
///
/// # Errors
///
/// Returns `FilterError::InvalidExpression` if parsing fails, the required feature is disabled,
/// or the expression cannot be converted into a typed filter node.
pub fn parse_odata_filter<F: FilterField>(raw: &str) -> FilterResult<FilterNode<F>> {
    #[cfg(feature = "with-odata-params")]
    {
        use odata_params::filters::parse_str;

        let ast = parse_str(raw).map_err(|e| FilterError::InvalidExpression(format!("{e:?}")))?;
        let ast: odata_ast::Expr = ast.into();
        convert_expr_to_filter_node::<F>(&ast)
    }

    #[cfg(not(feature = "with-odata-params"))]
    {
        let _ = raw;
        Err(FilterError::InvalidExpression(
            "OData filter parsing requires 'with-odata-params' feature".to_owned(),
        ))
    }
}

/// Convert a parsed `OData` AST expression into a typed `FilterNode`.
///
/// # Errors
///
/// Returns `FilterError` if the expression is invalid, references unknown fields, uses unsupported
/// operations, or contains type mismatches.
pub fn convert_expr_to_filter_node<F: FilterField>(
    expr: &odata_ast::Expr,
) -> FilterResult<FilterNode<F>> {
    use odata_ast::Expr as E;

    match expr {
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

        E::Compare(left, op, right) => {
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

            let field = F::from_name(field_name)
                .ok_or_else(|| FilterError::UnknownField(field_name.to_owned()))?;

            validate_value_type(field, &value)?;

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

        E::Function(func_name, args) => {
            let name_lower = func_name.to_ascii_lowercase();
            match (name_lower.as_str(), args.as_slice()) {
                (
                    "contains",
                    [
                        E::Identifier(field_name),
                        E::Value(odata_ast::Value::String(s)),
                    ],
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
                        FilterOp::Contains,
                        odata_ast::Value::String(s.clone()),
                    ))
                }
                (
                    "startswith",
                    [
                        E::Identifier(field_name),
                        E::Value(odata_ast::Value::String(s)),
                    ],
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
                    [
                        E::Identifier(field_name),
                        E::Value(odata_ast::Value::String(s)),
                    ],
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

        E::In(_left, _list) => Err(FilterError::UnsupportedOperation(
            "IN operator not yet supported in typed filters".to_owned(),
        )),

        E::Identifier(name) => Err(FilterError::BareIdentifier(name.clone())),
        E::Value(_) => Err(FilterError::BareLiteral),
    }
}

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
