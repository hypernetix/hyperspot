#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;
extern crate rustc_hir;

use rustc_ast::LitKind;
use rustc_hir::{Expr, ExprKind};
use rustc_lint::{LateContext, LateLintPass, LintContext};

dylint_linting::declare_late_lint! {
    /// ### What it does
    ///
    /// Detects hardcoded database and service connection strings in source code.
    ///
    /// ### Why is this bad?
    ///
    /// Hardcoded connection strings are a security and operational risk:
    /// - Connection strings often contain credentials (username/password)
    /// - Different environments need different connection strings
    /// - Secrets are exposed in version control history
    /// - Changes require code modifications and redeployment
    ///
    /// ### Example
    ///
    /// ```rust,ignore
    /// // Bad - hardcoded connection string
    /// let db_url = "postgres://user:password@localhost:5432/mydb";
    /// let cache = "redis://localhost:6379";
    /// ```
    ///
    /// Use instead:
    ///
    /// ```rust,ignore
    /// // Good - load from environment or config
    /// let db_url = std::env::var("DATABASE_URL")?;
    /// let cache = config.redis_url.clone();
    /// ```
    pub DE0407_NO_HARDCODED_CONNECTION_STRINGS,
    Warn,
    "avoid hardcoding database connection strings (DE0407)"
}

/// Connection string URL schemes that indicate hardcoded database/service URLs
const CONNECTION_SCHEMES: &[&str] = &[
    "postgres://",
    "postgresql://",
    "mysql://",
    "mariadb://",
    "mongodb://",
    "mongodb+srv://",
    "redis://",
    "rediss://",
    "amqp://",
    "amqps://",
    "nats://",
    "kafka://",
    "mssql://",
    "sqlserver://",
    "oracle://",
    "sqlite://",
];

impl<'tcx> LateLintPass<'tcx> for De0407NoHardcodedConnectionStrings {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'tcx>) {
        // Check string literals for connection string patterns
        let ExprKind::Lit(lit) = &expr.kind else { return };
        let LitKind::Str(symbol, _) = lit.node else { return };

        let string_value = symbol.as_str();

        // Check if it matches any connection scheme
        for scheme in CONNECTION_SCHEMES {
            if string_value.starts_with(scheme) {
                // Only flag if there's actual connection info after the scheme
                // (not just the scheme prefix used for URL detection)
                let after_scheme = &string_value[scheme.len()..];
                if !after_scheme.is_empty() && !after_scheme.starts_with('/') {
                    // Has host/path info - this is a real connection string
                    cx.span_lint(DE0407_NO_HARDCODED_CONNECTION_STRINGS, expr.span, |diag| {
                        diag.primary_message("hardcoded connection string detected (DE0407)");
                        diag.help("load connection strings from environment variables (std::env::var) or configuration files");
                        diag.note("hardcoded connection strings expose credentials and make environment-specific configuration difficult");
                    });
                    return;
                }
            }
        }
    }
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
            "DE0407",
            "hardcoded connection"
        );
    }
}
