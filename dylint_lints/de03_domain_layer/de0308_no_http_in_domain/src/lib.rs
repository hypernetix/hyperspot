#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;

use rustc_ast::{Item, ItemKind, UseTree, UseTreeKind};
use rustc_lint::{EarlyLintPass, LintContext};

use lint_utils::is_in_domain_path;

dylint_linting::declare_early_lint! {
    /// ### What it does
    ///
    /// Checks that domain modules do not reference HTTP types or status codes.
    ///
    /// ### Why is this bad?
    ///
    /// Domain modules should be transport-agnostic. HTTP is just one possible
    /// transport layer. Referencing HTTP types in domain code couples the business
    /// logic to a specific transport mechanism.
    ///
    /// ### Example
    ///
    /// ```rust,ignore
    /// // Bad - HTTP types in domain
    /// mod domain {
    ///     use http::StatusCode;
    ///     
    ///     pub fn check_result() -> StatusCode {
    ///         StatusCode::OK  // ❌ HTTP-specific
    ///     }
    /// }
    /// ```
    ///
    /// Use instead:
    ///
    /// ```rust,ignore
    /// // Good - domain errors converted in API layer
    /// mod domain {
    ///     pub enum DomainResult {
    ///         Success,
    ///         NotFound,
    ///         InvalidData,
    ///     }
    /// }
    /// ```
    pub DE0308_NO_HTTP_IN_DOMAIN,
    Deny,
    "domain modules should not reference HTTP types or status codes (DE0308)"
}

/// HTTP-related patterns forbidden in domain code
/// Only includes frameworks actually used in the project: axum, hyper, http
const HTTP_PATTERNS: &[&str] = &[
    "http::",
    "http::StatusCode",
    "http::Method",
    "http::HeaderMap",
    "http::Request",
    "http::Response",
    "axum::http",
    "hyper::StatusCode",
    "hyper::Method",
    "reqwest::StatusCode",
];

fn use_tree_to_string(tree: &UseTree) -> String {
    match &tree.kind {
        UseTreeKind::Simple(..) | UseTreeKind::Glob => {
            tree.prefix.segments.iter()
                .map(|seg| seg.ident.name.as_str())
                .collect::<Vec<_>>()
                .join("::")
        }
        UseTreeKind::Nested { items, .. } => {
            let prefix = tree.prefix.segments.iter()
                .map(|seg| seg.ident.name.as_str())
                .collect::<Vec<_>>()
                .join("::");
            
            for (nested_tree, _) in items {
                let nested_str = use_tree_to_string(nested_tree);
                if !nested_str.is_empty() {
                    return format!("{}::{}", prefix, nested_str);
                }
            }
            prefix
        }
    }
}

fn check_use_in_domain(cx: &rustc_lint::EarlyContext<'_>, item: &Item) {
    let ItemKind::Use(use_tree) = &item.kind else {
        return;
    };

    let path_str = use_tree_to_string(use_tree);
    for pattern in HTTP_PATTERNS {
        if path_str.starts_with(pattern) {
            cx.span_lint(DE0308_NO_HTTP_IN_DOMAIN, item.span, |diag| {
                diag.primary_message(
                    "domain module imports HTTP type (DE0308)"
                );
                diag.help("domain should be transport-agnostic; handle HTTP in api/ layer");
            });
            break;
        }
    }
}

impl EarlyLintPass for De0308NoHttpInDomain {
    fn check_item(&mut self, cx: &rustc_lint::EarlyContext<'_>, item: &Item) {
        // Check use statements in file-based domain modules
        if matches!(item.kind, ItemKind::Use(_))
            && is_in_domain_path(cx.sess().source_map(), item.span) {
            check_use_in_domain(cx, item);
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
        lint_utils::test_comment_annotations_match_stderr(&ui_dir, "DE0308", "HTTP in domain");
    }
}
