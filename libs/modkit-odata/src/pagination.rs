//! Filter hashing utilities for `OData` pagination

use crate::ast;
use sha2::{Digest, Sha256};

/// Normalize filter AST for consistent hashing
/// Produces a stable string representation for deterministic hashing
#[must_use]
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
                ast::Value::Null => "NULL".to_owned(),
                ast::Value::Bool(b) => format!("BOOL({b})"),
                ast::Value::Number(n) => format!("NUM({})", n.normalized()),
                ast::Value::Uuid(u) => {
                    format!("UUID({})", u.as_hyphenated().to_string().to_lowercase())
                }
                ast::Value::DateTime(dt) => format!("DATETIME({})", dt.to_rfc3339()),
                ast::Value::Date(d) => format!("DATE({})", d.format("%Y-%m-%d")),
                ast::Value::Time(t) => format!("TIME({})", t.format("%H:%M:%S%.f")),
                ast::Value::String(s) => format!("STR({s})"),
            },
        }
    }

    normalize_expr(expr)
}

/// Generate a short hash from a filter expression for cursor consistency checks
/// Returns a 16-character hex string (64-bit hash)
#[must_use]
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
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::ast::{CompareOperator, Expr, Value};

    #[test]
    fn test_normalize_filter_consistency() {
        // Test that the same logical filter produces the same normalized string
        let expr1 = Expr::Compare(
            Box::new(Expr::Identifier("name".to_owned())),
            CompareOperator::Eq,
            Box::new(Expr::Value(Value::String("test".to_owned()))),
        );

        let expr2 = Expr::Compare(
            Box::new(Expr::Identifier("name".to_owned())),
            CompareOperator::Eq,
            Box::new(Expr::Value(Value::String("test".to_owned()))),
        );

        assert_eq!(
            normalize_filter_for_hash(&expr1),
            normalize_filter_for_hash(&expr2)
        );
    }

    #[test]
    fn test_short_filter_hash_consistency() {
        let expr = Expr::Compare(
            Box::new(Expr::Identifier("id".to_owned())),
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
