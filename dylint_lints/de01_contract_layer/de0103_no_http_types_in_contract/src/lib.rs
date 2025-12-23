#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;

use rustc_ast::{Item, ItemKind, UseTree, UseTreeKind};
use rustc_lint::{EarlyLintPass, LintContext};

use lint_utils::is_in_contract_path;

dylint_linting::declare_early_lint! {
    /// ### What it does
    ///
    /// Checks that contract modules do not import HTTP-specific types.
    ///
    /// ### Why is this bad?
    ///
    /// Contract modules should be transport-agnostic. HTTP is just one possible
    /// transport layer. Using HTTP types in contracts couples the domain logic
    /// to a specific transport mechanism.
    ///
    /// ### Example
    ///
    /// ```rust
    /// // Bad - HTTP types in contract
    /// mod contract {
    ///     use http::StatusCode\;
    ///     
    ///     pub struct OrderResult {
    ///         pub status: StatusCode,  // ❌ HTTP-specific
    ///     }
    /// }
    /// ```
    ///
    /// Use instead:
    ///
    /// ```rust
    /// // Good - domain types in contract
    /// mod contract {
    ///     pub enum OrderStatus {
    ///         Pending,
    ///         Confirmed,
    ///         Shipped,
    ///     }
    ///     
    ///     pub struct OrderResult {
    ///         pub status: OrderStatus,  // ✅ Domain type
    ///     }
    /// }
    /// 
    /// // HTTP types in API layer
    /// mod api {
    ///     use http::StatusCode\;
    ///     // HTTP layer converts between HTTP and domain types
    /// }
    /// ```
    pub DE0103_NO_HTTP_TYPES_IN_CONTRACT,
    Deny,
    "contract modules should not reference HTTP-specific types (DE0103)"
}

const HTTP_TYPE_PATTERNS: &[&str] = &[
    "axum::http",
    "http::StatusCode",
    "http::Method",
    "http::HeaderMap",
    "http::HeaderName",
    "http::HeaderValue",
    "http::Request",
    "http::Response",
    "hyper::StatusCode",
    "hyper::Method",
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

fn check_use_in_contract(cx: &rustc_lint::EarlyContext<'_>, item: &Item) {
    let ItemKind::Use(use_tree) = &item.kind else {
        return;
    };

    let path_str = use_tree_to_string(use_tree);
    for pattern in HTTP_TYPE_PATTERNS {
        if path_str.contains(pattern) {
            cx.span_lint(DE0103_NO_HTTP_TYPES_IN_CONTRACT, item.span, |diag| {
                diag.primary_message(
                    "contract module imports HTTP type (DE0103)"
                );
                diag.help("contract modules should be transport-agnostic; move HTTP types to api/rest/");
            });
            break;
        }
    }
}

impl EarlyLintPass for De0103NoHttpTypesInContract {
    fn check_item(&mut self, cx: &rustc_lint::EarlyContext<'_>, item: &Item) {
        // Check use statements in file-based contract modules
        if matches!(item.kind, ItemKind::Use(_))
            && is_in_contract_path(cx.sess().source_map(), item.span) {
            check_use_in_contract(cx, item);
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
        lint_utils::test_comment_annotations_match_stderr(&ui_dir, "DE0103", "HTTP types in contract");
    }
}
