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
    /// Detects hardcoded secrets such as API keys, passwords, tokens, and other sensitive
    /// credentials in string literals.
    ///
    /// ### Why is this bad?
    ///
    /// Hardcoded secrets in source code are a serious security vulnerability:
    /// - Secrets are stored in version control history
    /// - Anyone with repository access can see them
    /// - Secrets can't be rotated without code changes
    /// - Risk of accidental exposure in public repositories
    ///
    /// ### Example
    ///
    /// ```rust,ignore
    /// // Bad - hardcoded API key
    /// let api_key = "sk_live_51H1234567890abcdef";
    /// let password = "MySecretP@ssw0rd";
    /// let token = "ghp_1234567890abcdefghijklmnopqrstuv";
    /// ```
    ///
    /// Use instead:
    ///
    /// ```rust,ignore
    /// // Good - load from environment or config
    /// let api_key = std::env::var("API_KEY")?;
    /// let config = load_config()?;
    /// let token = config.auth_token;
    /// ```
    pub DE0703_NO_HARDCODED_SECRETS,
    Deny,
    "avoid hardcoding secrets like API keys, passwords, or tokens (DE0703)"
}

impl<'tcx> LateLintPass<'tcx> for De0703NoHardcodedSecrets {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'tcx>) {
        // Check string literals for potential secrets
        let ExprKind::Lit(lit) = &expr.kind else { return };
        let LitKind::Str(symbol, _) = lit.node else { return };

        let string_value = symbol.as_str();

        // Skip short strings
        if string_value.len() < 8 {
            return;
        }

        // Check for common secret patterns
        if is_potential_secret(string_value) {
            cx.span_lint(DE0703_NO_HARDCODED_SECRETS, expr.span, |diag| {
                diag.primary_message("potential hardcoded secret detected (DE0703)");
                diag.help("load secrets from environment variables (std::env::var) or configuration files");
                diag.note("hardcoded secrets are a security risk and should never be committed to version control");
            });
        }
    }
}

fn is_potential_secret(s: &str) -> bool {
    let lower = s.to_lowercase();
    
    // Skip example code snippets and documentation (but not parts of keys like "AKIAEXAMPLE")
    if lower.contains("e.g.") || lower.contains("for example") || lower.starts_with("example") {
        return false;
    }
    
    // Skip SQL queries
    let sql_keywords = ["select ", "insert ", "update ", "delete ", "create ", "alter ", "drop "];
    for keyword in &sql_keywords {
        if lower.starts_with(keyword) || lower.contains(&format!(" {}", keyword)) {
            return false;
        }
    }
    
    // Common secret patterns
    let patterns = [
        ("api_key", "="),
        ("apikey", "="),
        ("api-key", "="),
        ("password", "="),
        ("passwd", "="),
        ("secret", "="),
        ("token", "="),
        ("auth", "="),
        ("credential", "="),
    ];
    
    // Check for key=value patterns
    for (key, sep) in &patterns {
        if lower.contains(key) && lower.contains(sep) {
            return true;
        }
    }
    
    // Check for common API key prefixes
    let prefixes = [
        "sk_live_",   // Stripe live key
        "sk_test_",   // Stripe test key  
        "pk_live_",   // Stripe public live
        "pk_test_",   // Stripe public test
        "ghp_",       // GitHub personal token
        "gho_",       // GitHub OAuth token
        "ghu_",       // GitHub user token
        "ghs_",       // GitHub server token
        "ghr_",       // GitHub refresh token
        "AKIA",       // AWS access key
        "ASIA",       // AWS temporary access key
        "ya29.",      // Google OAuth
        "AIza",       // Google API key
    ];
    
    for prefix in &prefixes {
        if s.starts_with(prefix) {
            return true;
        }
    }
    
    // Check for high-entropy strings that might be secrets
    // (at least 20 chars, mix of alphanumeric)
    if s.len() >= 20 && has_high_entropy(s) {
        return true;
    }
    
    false
}

fn has_high_entropy(s: &str) -> bool {
    // Skip if contains common English words (likely normal text)
    let lower = s.to_lowercase();
    let common_words = [
        "processing", "request", "response", "user", "data", "message",
        "error", "success", "failed", "loading", "please", "welcome",
        "hello", "example", "test", "debug", "info", "warning"
    ];
    
    for word in &common_words {
        if lower.contains(word) {
            return false;
        }
    }
    
    let mut has_upper = false;
    let mut has_lower = false;
    let mut has_digit = false;
    let mut unique_chars = std::collections::HashSet::new();
    
    for c in s.chars() {
        unique_chars.insert(c);
        if c.is_uppercase() { has_upper = true; }
        if c.is_lowercase() { has_lower = true; }
        if c.is_numeric() { has_digit = true; }
    }
    
    // High entropy: mix of upper, lower, digits and very high uniqueness
    let char_types = [has_upper, has_lower, has_digit]
        .iter()
        .filter(|&&x| x)
        .count();
    
    let uniqueness_ratio = unique_chars.len() as f64 / s.len() as f64;
    
    // Require all 3 types (upper, lower, digit) and very high uniqueness
    char_types == 3 && uniqueness_ratio > 0.7
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
            "DE0703",
            "hardcoded secret"
        );
    }
}
