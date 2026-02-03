#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;

use lint_utils::{is_in_contract_module_ast, is_in_domain_path, use_tree_to_strings};
use rustc_ast::{Item, ItemKind, Ty, TyKind};
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};

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
    "http",
    "axum",
    "hyper",
];

/// Check if a path matches an HTTP pattern.
/// Returns true only if path equals pattern exactly or starts with "pattern::"
/// This avoids false positives like "http_client" matching "http".
fn matches_http_pattern(path: &str) -> Option<&'static str> {
    for pattern in HTTP_PATTERNS {
        if path == *pattern || path.starts_with(&format!("{pattern}::")) {
            return Some(pattern);
        }
    }
    None
}

fn check_use_item(cx: &EarlyContext<'_>, item: &Item, tree: &rustc_ast::UseTree) {
    for path_str in use_tree_to_strings(tree) {
        if let Some(pattern) = matches_http_pattern(&path_str) {
            cx.span_lint(DE0308_NO_HTTP_IN_DOMAIN, item.span, |diag| {
                diag.primary_message(format!(
                    "domain module imports HTTP type `{pattern}` (DE0308)"
                ));
                diag.help("domain should be transport-agnostic; handle HTTP in api/ layer");
            });
            return;
        }
    }
}

fn check_type_in_domain(cx: &rustc_lint::EarlyContext<'_>, ty: &Ty) {
    match &ty.kind {
        TyKind::Path(_, path) => {
            // Check the path itself
            let path_str = path
                .segments
                .iter()
                .map(|seg| seg.ident.name.as_str())
                .collect::<Vec<_>>()
                .join("::");

            if matches_http_pattern(&path_str).is_some() {
                cx.span_lint(DE0308_NO_HTTP_IN_DOMAIN, ty.span, |diag| {
                    diag.primary_message(format!(
                        "domain module uses HTTP type `{}` (DE0308)",
                        path_str
                    ));
                    diag.help("domain should be transport-agnostic; handle HTTP in api/ layer");
                });
                return;
            }

            // Recursively check generic arguments (e.g., Option<http::StatusCode>)
            for segment in &path.segments {
                if let Some(args) = &segment.args {
                    if let rustc_ast::GenericArgs::AngleBracketed(ref angle_args) = **args {
                        for arg in &angle_args.args {
                            if let rustc_ast::AngleBracketedArg::Arg(rustc_ast::GenericArg::Type(
                                inner_ty,
                            )) = arg
                            {
                                check_type_in_domain(cx, inner_ty);
                            }
                        }
                    }
                }
            }
        }
        // Handle references: &http::Request
        TyKind::Ref(_, mut_ty) => {
            check_type_in_domain(cx, &mut_ty.ty);
        }
        // Handle slices: [http::StatusCode]
        TyKind::Slice(inner_ty) => {
            check_type_in_domain(cx, inner_ty);
        }
        // Handle arrays: [http::StatusCode; 10]
        TyKind::Array(inner_ty, _) => {
            check_type_in_domain(cx, inner_ty);
        }
        // Handle raw pointers: *const http::Request
        TyKind::Ptr(mut_ty) => {
            check_type_in_domain(cx, &mut_ty.ty);
        }
        // Handle tuples: (http::Request, String)
        TyKind::Tup(types) => {
            for inner_ty in types {
                check_type_in_domain(cx, inner_ty);
            }
        }
        // Handle trait objects: dyn http::Service
        TyKind::TraitObject(bounds, _) => {
            for bound in bounds {
                if let rustc_ast::GenericBound::Trait(trait_ref) = bound {
                    // Check the trait path itself
                    let path = &trait_ref.trait_ref.path;
                    let path_str = path
                        .segments
                        .iter()
                        .map(|seg| seg.ident.name.as_str())
                        .collect::<Vec<_>>()
                        .join("::");

                    if matches_http_pattern(&path_str).is_some() {
                        cx.span_lint(DE0308_NO_HTTP_IN_DOMAIN, ty.span, |diag| {
                            diag.primary_message(format!(
                                "domain module uses HTTP trait `{}` (DE0308)",
                                path_str
                            ));
                            diag.help(
                                "domain should be transport-agnostic; handle HTTP in api/ layer",
                            );
                        });
                        return;
                    }
                }
            }
        }
        // Handle impl Trait: impl http::Service
        TyKind::ImplTrait(_, bounds) => {
            for bound in bounds {
                if let rustc_ast::GenericBound::Trait(trait_ref) = bound {
                    // Check the trait path itself
                    let path = &trait_ref.trait_ref.path;
                    let path_str = path
                        .segments
                        .iter()
                        .map(|seg| seg.ident.name.as_str())
                        .collect::<Vec<_>>()
                        .join("::");

                    if matches_http_pattern(&path_str).is_some() {
                        cx.span_lint(DE0308_NO_HTTP_IN_DOMAIN, ty.span, |diag| {
                            diag.primary_message(format!(
                                "domain module uses HTTP trait `{}` (DE0308)",
                                path_str
                            ));
                            diag.help(
                                "domain should be transport-agnostic; handle HTTP in api/ layer",
                            );
                        });
                        return;
                    }
                }
            }
        }
        _ => {}
    }
}

impl EarlyLintPass for De0308NoHttpInDomain {
    fn check_item(&mut self, cx: &rustc_lint::EarlyContext<'_>, item: &Item) {
        // Skip if not in domain path or if in contract module (contracts can have HTTP types)
        if !is_in_domain_path(cx.sess().source_map(), item.span)
            || is_in_contract_module_ast(cx, item)
        {
            return;
        }

        match &item.kind {
            // Check use statements
            ItemKind::Use(use_tree) => {
                check_use_item(cx, item, use_tree);
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
