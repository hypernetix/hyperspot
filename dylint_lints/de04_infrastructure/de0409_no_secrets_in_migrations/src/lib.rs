#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;
extern crate rustc_hir;
extern crate rustc_span;

use rustc_ast::LitKind;
use rustc_hir::{Expr, ExprKind};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_span::FileName;

dylint_linting::declare_late_lint! {
    /// ### What it does
    ///
    /// Detects hardcoded secrets (passwords, API keys, tokens) in database migration files.
    ///
    /// ### Why is this bad?
    ///
    /// Migration files are often committed to version control and may be executed
    /// in different environments. Hardcoding secrets in migrations:
    /// - Exposes credentials in version control history
    /// - Makes it impossible to use different secrets per environment
    /// - Creates a security vulnerability if the repository is public
    ///
    /// ### Example
    ///
    /// ```rust,ignore
    /// // Bad - hardcoded password in migration
    /// // File: migrations/20240101_add_admin.rs
    /// fn up(conn: &Connection) {
    ///     conn.execute("INSERT INTO users (name, password) VALUES ('admin', 'secret123')");
    /// }
    /// ```
    ///
    /// Use instead:
    ///
    /// ```rust,ignore
    /// // Good - use environment variables or config
    /// // File: migrations/20240101_add_admin.rs
    /// fn up(conn: &Connection) {
    ///     let password = std::env::var("ADMIN_PASSWORD")?;
    ///     conn.execute(&format!("INSERT INTO users (name, password) VALUES ('admin', $1)"), &[&password]);
    /// }
    /// ```
    pub DE0409_NO_SECRETS_IN_MIGRATIONS,
    Warn,
    "avoid hardcoding secrets in database migration files (DE0409)"
}

/// Secret patterns to detect in migration files
const SECRET_PATTERNS: &[(&str, &str)] = &[
    ("password", "="),
    ("passwd", "="),
    ("secret", "="),
    ("api_key", "="),
    ("apikey", "="),
    ("api-key", "="),
    ("token", "="),
    ("auth_token", "="),
    ("access_token", "="),
    ("private_key", "="),
    ("credential", "="),
];

/// API key prefixes that indicate hardcoded secrets
const SECRET_PREFIXES: &[&str] = &[
    "sk_live_",
    "sk_test_",
    "pk_live_",
    "pk_test_",
    "ghp_",
    "gho_",
    "AKIA",
    "ASIA",
];

fn is_in_migrations_path(cx: &LateContext<'_>, span: rustc_span::Span) -> bool {
    let file_name = cx.sess().source_map().span_to_filename(span);

    let path_str = match &file_name {
        FileName::Real(real_name) => {
            real_name.local_path()
                .map(|p| p.to_string_lossy().to_string())
        }
        _ => None,
    };

    let Some(path_str) = path_str else { return false };

    // Check for simulated directory in test files
    if let Some(simulated) = extract_simulated_dir(&path_str) {
        return simulated.contains("/migrations/") || simulated.contains("\\migrations\\");
    }

    path_str.contains("/migrations/") || path_str.contains("\\migrations\\")
}

fn extract_simulated_dir(path_str: &str) -> Option<String> {
    // Only check for simulated_dir in temporary paths
    let is_temp = path_str.contains("/tmp/")
        || path_str.contains("/var/folders/")
        || path_str.contains("\\Temp\\")
        || path_str.contains(".tmp");

    if !is_temp {
        return None;
    }

    let contents = std::fs::read_to_string(std::path::PathBuf::from(path_str)).ok()?;

    for line in contents.lines().take(1) {
        let trimmed = line.trim();
        if trimmed.starts_with("// simulated_dir=") {
            return Some(trimmed.trim_start_matches("// simulated_dir=").to_string());
        }
    }

    None
}

fn is_potential_secret(s: &str) -> bool {
    let lower = s.to_lowercase();

    // Check for key=value patterns
    for (key, sep) in SECRET_PATTERNS {
        if lower.contains(key) && lower.contains(sep) {
            return true;
        }
    }

    // Check for common API key prefixes
    for prefix in SECRET_PREFIXES {
        if s.starts_with(prefix) {
            return true;
        }
    }

    // Check for connection strings with credentials
    if (lower.contains("://") && lower.contains("@")) &&
       (lower.contains("postgres") || lower.contains("mysql") ||
        lower.contains("mongodb") || lower.contains("redis")) {
        return true;
    }

    false
}

impl<'tcx> LateLintPass<'tcx> for De0409NoSecretsInMigrations {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'tcx>) {
        // Only check migration files
        if !is_in_migrations_path(cx, expr.span) {
            return;
        }

        // Check string literals for potential secrets
        let ExprKind::Lit(lit) = &expr.kind else { return };
        let LitKind::Str(symbol, _) = lit.node else { return };

        let string_value = symbol.as_str();

        // Skip short strings
        if string_value.len() < 8 {
            return;
        }

        if is_potential_secret(string_value) {
            cx.span_lint(DE0409_NO_SECRETS_IN_MIGRATIONS, expr.span, |diag| {
                diag.primary_message("potential secret detected in migration file (DE0409)");
                diag.help("load secrets from environment variables or use parameterized queries");
                diag.note("migration files are committed to version control and may be shared");
            });
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
            "DE0409",
            "secret in migration"
        );
    }
}
