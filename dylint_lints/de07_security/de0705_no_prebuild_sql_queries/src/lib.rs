#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;
extern crate rustc_hir;
extern crate rustc_span;

use clippy_utils::macros::root_macro_call_first_node;
use rustc_ast::LitKind;
use rustc_hir::{Expr, ExprKind};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_span::sym;

dylint_linting::declare_late_lint! {
    /// ### What it does
    ///
    /// Detects prebuilt SQL queries where strings are concatenated or formatted
    /// instead of using parameterized queries.
    ///
    /// ### Why is this bad?
    ///
    /// Prebuilt SQL queries create security vulnerabilities:
    /// - Attackers can inject malicious SQL through user input
    /// - Can lead to data breaches and authentication bypass
    /// - Parameterized queries are the industry standard for SQL safety
    ///
    /// ### Example
    ///
    /// ```rust,ignore
    /// // Bad - prebuilt query with string formatting
    /// let query = format!("SELECT * FROM users WHERE id = {}", user_input);
    /// let query = "SELECT * FROM users WHERE name = '".to_string() + name + "'";
    /// ```
    ///
    /// Use instead:
    ///
    /// ```rust,ignore
    /// // Good - use parameterized queries
    /// sqlx::query("SELECT * FROM users WHERE id = $1").bind(user_input);
    /// conn.execute("SELECT * FROM users WHERE name = ?", &[name])?;
    /// ```
    pub DE0705_NO_PREBUILD_SQL_QUERIES,
    Warn,
    "avoid prebuilt SQL queries; use parameterized queries instead (DE0705)"
}

/// SQL keywords that indicate a query string
const SQL_KEYWORDS: &[&str] = &[
    "SELECT ",
    "INSERT ",
    "UPDATE ",
    "DELETE ",
    "DROP ",
    "CREATE ",
    "ALTER ",
    "TRUNCATE ",
    "EXEC ",
    "EXECUTE ",
];

fn is_sql_query(s: &str) -> bool {
    let upper = s.to_uppercase();
    SQL_KEYWORDS.iter().any(|kw| upper.contains(kw))
}

impl<'tcx> LateLintPass<'tcx> for De0705NoPrebuildSqlQueries {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'tcx>) {
        // Check for format! macro with SQL content
        if let Some(macro_call) = root_macro_call_first_node(cx, expr) {
            if cx.tcx.is_diagnostic_item(sym::format_macro, macro_call.def_id) {
                // Check if the format string contains SQL keywords
                if let ExprKind::Call(_, args) = &expr.kind {
                    if let Some(first_arg) = args.first() {
                        if let Some(sql_string) = extract_format_string(cx, first_arg) {
                            if is_sql_query(&sql_string) && sql_string.contains("{}") {
                                cx.span_lint(DE0705_NO_PREBUILD_SQL_QUERIES, macro_call.span, |diag| {
                                    diag.primary_message("prebuilt SQL query detected: format! with SQL query (DE0705)");
                                    diag.help("use parameterized queries with .bind() instead of string formatting");
                                    diag.note("prebuilt queries are vulnerable to SQL injection attacks");
                                });
                            }
                        }
                    }
                }
            }
        }

        // Check for string concatenation with SQL
        if let ExprKind::Binary(op, left, right) = &expr.kind {
            if matches!(op.node, rustc_hir::BinOpKind::Add) {
                // Check if either side contains SQL keywords
                let left_sql = extract_string_literal(cx, left).map(|s| is_sql_query(&s)).unwrap_or(false);
                let right_sql = extract_string_literal(cx, right).map(|s| is_sql_query(&s)).unwrap_or(false);

                if left_sql || right_sql {
                    cx.span_lint(DE0705_NO_PREBUILD_SQL_QUERIES, expr.span, |diag| {
                        diag.primary_message("prebuilt SQL query detected: string concatenation with SQL query (DE0705)");
                        diag.help("use parameterized queries with .bind() instead of string concatenation");
                        diag.note("prebuilt queries are vulnerable to SQL injection attacks");
                    });
                }
            }
        }

        // Check for .to_string() + on SQL strings (method call chains)
        if let ExprKind::MethodCall(method, receiver, _, _) = &expr.kind {
            if method.ident.name.as_str() == "to_string" {
                if let Some(sql_string) = extract_string_literal(cx, receiver) {
                    if is_sql_query(&sql_string) {
                        // Check if this to_string is followed by concatenation
                        // We'll flag the to_string() call if the SQL string looks like it expects concatenation
                        if sql_string.ends_with('\'') || sql_string.ends_with('=') || sql_string.ends_with(' ') {
                            cx.span_lint(DE0705_NO_PREBUILD_SQL_QUERIES, expr.span, |diag| {
                                diag.primary_message("prebuilt SQL query detected: SQL string converted for concatenation (DE0705)");
                                diag.help("use parameterized queries with .bind() instead of string concatenation");
                                diag.note("prebuilt queries are vulnerable to SQL injection attacks");
                            });
                        }
                    }
                }
            }
        }
    }
}

fn extract_format_string<'tcx>(cx: &LateContext<'tcx>, expr: &'tcx Expr<'tcx>) -> Option<String> {
    // Try to extract the format string from format_args! expansion
    // This is complex due to macro expansion, so we do a simplified check
    if let ExprKind::Call(func, args) = &expr.kind {
        if let ExprKind::Path(qpath) = &func.kind {
            let path_str = format!("{:?}", qpath);
            if path_str.contains("format_args") {
                if let Some(first_arg) = args.first() {
                    return extract_string_literal(cx, first_arg);
                }
            }
        }
    }
    None
}

fn extract_string_literal<'tcx>(_cx: &LateContext<'tcx>, expr: &'tcx Expr<'tcx>) -> Option<String> {
    if let ExprKind::Lit(lit) = &expr.kind {
        if let LitKind::Str(symbol, _) = lit.node {
            return Some(symbol.as_str().to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    #[test]
    fn ui_examples() {
        dylint_testing::ui_test_examples(env!("CARGO_PKG_NAME"));
    }

    #[test]
    fn test_comment_annotations_match_stderr() {
        let ui_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui");
        lint_utils::test_comment_annotations_match_stderr(
            &ui_dir,
            "DE0705",
            "prebuilt SQL query"
        );
    }
}
