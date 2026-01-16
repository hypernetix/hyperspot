#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;

use rustc_ast::{Item, ItemKind, UseTree, UseTreeKind, Ty, TyKind};
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

fn use_tree_to_strings(tree: &UseTree) -> Vec<String> {
    match &tree.kind {
        UseTreeKind::Simple(..) | UseTreeKind::Glob => {
            vec![tree.prefix.segments.iter()
                .map(|seg| seg.ident.name.as_str())
                .collect::<Vec<_>>()
                .join("::")]
        }
        UseTreeKind::Nested { items, .. } => {
            let prefix = tree.prefix.segments.iter()
                .map(|seg| seg.ident.name.as_str())
                .collect::<Vec<_>>()
                .join("::");
            
            let mut paths = Vec::new();
            for (nested_tree, _) in items {
                for nested_str in use_tree_to_strings(nested_tree) {
                    if nested_str.is_empty() {
                        paths.push(prefix.clone());
                    } else if prefix.is_empty() {
                        paths.push(nested_str);
                    } else {
                        paths.push(format!("{}::{}", prefix, nested_str));
                    }
                }
            }
            if paths.is_empty() { vec![prefix] } else { paths }
        }
    }
}

fn check_use_in_domain(cx: &rustc_lint::EarlyContext<'_>, item: &Item) {
    let ItemKind::Use(use_tree) = &item.kind else {
        return;
    };

    for path_str in use_tree_to_strings(use_tree) {
        for pattern in HTTP_PATTERNS {
            if path_str.starts_with(pattern) {
                cx.span_lint(DE0308_NO_HTTP_IN_DOMAIN, item.span, |diag| {
                    diag.primary_message(
                        "domain module imports HTTP type (DE0308)"
                    );
                    diag.help("domain should be transport-agnostic; handle HTTP in api/ layer");
                });
                return;
            }
        }
    }
}

fn type_to_string(ty: &Ty) -> Option<String> {
    match &ty.kind {
        TyKind::Path(_, path) => {
            let path_str = path.segments.iter()
                .map(|seg| seg.ident.name.as_str())
                .collect::<Vec<_>>()
                .join("::");
            Some(path_str)
        }
        _ => None,
    }
}

fn check_type_in_domain(cx: &rustc_lint::EarlyContext<'_>, ty: &Ty) {
    if let Some(type_path) = type_to_string(ty) {
        for pattern in HTTP_PATTERNS {
            if type_path.starts_with(pattern) {
                cx.span_lint(DE0308_NO_HTTP_IN_DOMAIN, ty.span, |diag| {
                    diag.primary_message(
                        format!("domain module uses HTTP type `{}` (DE0308)", type_path)
                    );
                    diag.help("domain should be transport-agnostic; handle HTTP in api/ layer");
                });
                return;
            }
        }
    }
}

impl EarlyLintPass for De0308NoHttpInDomain {
    fn check_item(&mut self, cx: &rustc_lint::EarlyContext<'_>, item: &Item) {
        if !is_in_domain_path(cx.sess().source_map(), item.span) {
            return;
        }

        match &item.kind {
            // Check use statements
            ItemKind::Use(_) => {
                check_use_in_domain(cx, item);
            }
            // Check struct fields
            ItemKind::Struct(_, _, variant_data) => {
                for field in variant_data.fields() {
                    check_type_in_domain(cx, &field.ty);
                }
            }
            // Check enum variants
            ItemKind::Enum(_, _, enum_def) => {
                for variant in &enum_def.variants {
                    for field in variant.data.fields() {
                        check_type_in_domain(cx, &field.ty);
                    }
                }
            }
            // Check function signatures
            ItemKind::Fn(fn_item) => {
                // Check parameters
                for param in &fn_item.sig.decl.inputs {
                    check_type_in_domain(cx, &param.ty);
                }
                // Check return type
                if let rustc_ast::FnRetTy::Ty(ret_ty) = &fn_item.sig.decl.output {
                    check_type_in_domain(cx, ret_ty);
                }
            }
            // Check type aliases
            ItemKind::TyAlias(ty_alias) => {
                if let Some(ty) = &ty_alias.ty {
                    check_type_in_domain(cx, ty);
                }
            }
            _ => {}
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
