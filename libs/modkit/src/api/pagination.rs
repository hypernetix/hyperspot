//! OData pagination error handling and filter hashing utilities

use crate::api::problem::{Problem, ProblemResponse};
use axum::http::StatusCode;
use hex;
use odata_core::{ast, ODataPageError};
use sha2::{Digest, Sha256};

/// Map ODataPageError to RFC 9457 Problem once, so feature handlers don't do it
pub fn odata_page_error_to_problem(e: &ODataPageError, instance: &str) -> ProblemResponse {
    match e {
        ODataPageError::OrderMismatch => {
            Problem::new(StatusCode::BAD_REQUEST, "Order Mismatch", "ORDER_MISMATCH")
                .with_code("ORDER_MISMATCH")
                .with_instance(instance)
                .into()
        }
        ODataPageError::FilterMismatch => Problem::new(
            StatusCode::BAD_REQUEST,
            "Filter Mismatch",
            "FILTER_MISMATCH",
        )
        .with_code("FILTER_MISMATCH")
        .with_instance(instance)
        .into(),
        ODataPageError::InvalidCursor => {
            Problem::new(StatusCode::BAD_REQUEST, "Invalid Cursor", "INVALID_CURSOR")
                .with_code("INVALID_CURSOR")
                .with_instance(instance)
                .into()
        }
        ODataPageError::InvalidFilter(msg) => Problem::new(
            StatusCode::BAD_REQUEST,
            "Filter error",
            format!("invalid $filter: {}", msg),
        )
        .with_type("https://errors.example.com/ODATA_FILTER_INVALID")
        .with_code("ODATA_FILTER_INVALID")
        .with_instance(instance)
        .into(),
        ODataPageError::InvalidOrderByField(f) => Problem::new(
            StatusCode::BAD_REQUEST,
            "Unsupported OrderBy Field",
            format!("unsupported $orderby field: {}", f),
        )
        .with_code("UNSUPPORTED_ORDERBY_FIELD")
        .with_instance(instance)
        .into(),
        ODataPageError::InvalidLimit => Problem::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Invalid Limit",
            "INVALID_LIMIT",
        )
        .with_code("INVALID_LIMIT")
        .with_instance(instance)
        .into(),
        ODataPageError::OrderWithCursor => Problem::new(
            StatusCode::BAD_REQUEST,
            "Order With Cursor",
            "Cannot specify both $orderby and cursor parameters",
        )
        .with_code("ORDER_WITH_CURSOR")
        .with_instance(instance)
        .into(),
        ODataPageError::Db(_) => Problem::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Database Error",
            "An internal database error occurred",
        )
        .with_code("INTERNAL_DB")
        .with_instance(instance)
        .into(),
    }
}

/// Normalize filter AST for consistent hashing
/// Produces a stable string representation for deterministic hashing
pub fn normalize_filter_for_hash(expr: &ast::Expr) -> String {
    fn normalize_expr(expr: &ast::Expr) -> String {
        match expr {
            ast::Expr::And(left, right) => {
                format!("AND({},{})", normalize_expr(left), normalize_expr(right))
            }
            ast::Expr::Or(left, right) => {
                format!("OR({},{})", normalize_expr(left), normalize_expr(right))
            }
            ast::Expr::Not(inner) => {
                format!("NOT({})", normalize_expr(inner))
            }
            ast::Expr::Compare(left, op, right) => {
                let op_str = match op {
                    ast::CompareOperator::Eq => "EQ",
                    ast::CompareOperator::Ne => "NE",
                    ast::CompareOperator::Gt => "GT",
                    ast::CompareOperator::Ge => "GE",
                    ast::CompareOperator::Lt => "LT",
                    ast::CompareOperator::Le => "LE",
                };
                format!(
                    "CMP({},{},{})",
                    normalize_expr(left),
                    op_str,
                    normalize_expr(right)
                )
            }
            ast::Expr::In(expr, list) => {
                let list_str = list
                    .iter()
                    .map(normalize_expr)
                    .collect::<Vec<_>>()
                    .join(",");
                format!("IN({},{})", normalize_expr(expr), list_str)
            }
            ast::Expr::Function(name, args) => {
                let args_str = args
                    .iter()
                    .map(normalize_expr)
                    .collect::<Vec<_>>()
                    .join(",");
                format!("FN({},{})", name.to_lowercase(), args_str)
            }
            ast::Expr::Identifier(name) => {
                format!("ID({})", name.to_lowercase())
            }
            ast::Expr::Value(value) => match value {
                ast::Value::Null => "NULL".to_string(),
                ast::Value::Bool(b) => format!("BOOL({})", b),
                ast::Value::Number(n) => format!("NUM({})", n.normalized()),
                ast::Value::Uuid(u) => {
                    format!("UUID({})", u.as_hyphenated().to_string().to_lowercase())
                }
                ast::Value::DateTime(dt) => format!("DATETIME({})", dt.to_rfc3339()),
                ast::Value::Date(d) => format!("DATE({})", d.format("%Y-%m-%d")),
                ast::Value::Time(t) => format!("TIME({})", t.format("%H:%M:%S%.f")),
                ast::Value::String(s) => format!("STR({})", s),
            },
        }
    }

    normalize_expr(expr)
}

/// Generate a short hash from a filter expression for cursor consistency checks
/// Returns a 16-character hex string (64-bit hash)
pub fn short_filter_hash(expr: Option<&ast::Expr>) -> Option<String> {
    expr.map(|e| {
        let normalized = normalize_filter_for_hash(e);
        let mut hasher = Sha256::new();
        hasher.update(normalized.as_bytes());
        let bytes = hasher.finalize();
        hex::encode(&bytes[..8]) // Take first 8 bytes for 64-bit hash
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use odata_core::ast::{CompareOperator, Expr, Value};

    #[test]
    fn test_normalize_filter_consistency() {
        // Test that the same logical filter produces the same normalized string
        let expr1 = Expr::Compare(
            Box::new(Expr::Identifier("name".to_string())),
            CompareOperator::Eq,
            Box::new(Expr::Value(Value::String("test".to_string()))),
        );

        let expr2 = Expr::Compare(
            Box::new(Expr::Identifier("name".to_string())),
            CompareOperator::Eq,
            Box::new(Expr::Value(Value::String("test".to_string()))),
        );

        assert_eq!(
            normalize_filter_for_hash(&expr1),
            normalize_filter_for_hash(&expr2)
        );
    }

    #[test]
    fn test_short_filter_hash_consistency() {
        let expr = Expr::Compare(
            Box::new(Expr::Identifier("id".to_string())),
            CompareOperator::Gt,
            Box::new(Expr::Value(Value::Number(42.into()))),
        );

        let hash1 = short_filter_hash(Some(&expr));
        let hash2 = short_filter_hash(Some(&expr));

        assert_eq!(hash1, hash2);
        assert!(hash1.is_some());
        assert_eq!(hash1.as_ref().unwrap().len(), 16); // 8 bytes = 16 hex chars
    }

    #[test]
    fn test_short_filter_hash_none() {
        assert_eq!(short_filter_hash(None), None);
    }
}
