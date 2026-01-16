#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;

use rustc_ast::{Item, ItemKind, UseTree, UseTreeKind};
use rustc_lint::{EarlyLintPass, LintContext};

use lint_utils::is_in_domain_path;

dylint_linting::declare_early_lint! {
    /// ### What it does
    ///
    /// Checks that domain modules do not import infrastructure dependencies.
    ///
    /// ### Why is this bad?
    ///
    /// Domain modules should contain pure business logic and depend only on abstractions (ports),
    /// not concrete implementations. Importing infrastructure code (database, HTTP, external APIs)
    /// violates the Dependency Inversion Principle and makes domain logic harder to test.
    ///
    /// ### Example
    ///
    /// ```rust
    /// // Bad - infrastructure imports in domain
    /// mod domain {
    ///     use crate::infra::storage::UserRepository;  // ❌ concrete implementation
    ///     use sea_orm::*;  // ❌ database framework
    ///     use sqlx::*;     // ❌ database driver
    /// }
    /// ```
    ///
    /// Use instead:
    ///
    /// ```rust
    /// // Good - domain depends on abstractions
    /// mod domain {
    ///     use std::sync::Arc;
    ///     
    ///     pub trait UsersRepository: Send + Sync {
    ///         async fn find_by_id(&self, id: Uuid) -> Result<User, DomainError>;
    ///     }
    ///     
    ///     pub struct Service {
    ///         repo: Arc<dyn UsersRepository>,  // ✅ trait object
    ///     }
    /// }
    /// ```
    pub DE0301_NO_INFRA_IN_DOMAIN,
    Deny,
    "domain modules should not import infrastructure dependencies (DE0301)"
}

/// Forbidden import patterns for domain layer
const INFRA_PATTERNS: &[&str] = &[
    // Infrastructure layer
    "crate::infra",
    "crate::infrastructure",
    // Database frameworks (direct access forbidden, use modkit_db abstractions instead)
    "sea_orm",
    "sqlx",
    // HTTP/Web frameworks (only used ones: axum, hyper, http)
    "axum",
    "hyper",
    "http::",
    // API layer
    "crate::api",
    // External service clients  
    "reqwest",
    "tonic",
    // File system (should be abstracted)
    "std::fs",
    "tokio::fs",
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
    for pattern in INFRA_PATTERNS {
        if path_str.starts_with(pattern) {
            cx.span_lint(DE0301_NO_INFRA_IN_DOMAIN, item.span, |diag| {
                diag.primary_message(
                    format!("domain module imports infrastructure dependency `{}` (DE0301)", pattern)
                );
                diag.help("domain should depend only on abstractions; move infrastructure code to infra/ layer");
            });
            break;
        }
    }
}

impl EarlyLintPass for De0301NoInfraInDomain {
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
        lint_utils::test_comment_annotations_match_stderr(&ui_dir, "DE0301", "infra in domain");
    }
}
