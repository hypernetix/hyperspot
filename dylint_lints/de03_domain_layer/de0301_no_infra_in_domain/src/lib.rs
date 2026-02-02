#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;

use lint_utils::{is_in_domain_path, use_tree_to_strings};
use rustc_ast::{Item, ItemKind, Ty, TyKind};
use rustc_lint::{EarlyLintPass, LintContext};

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

fn check_use_in_domain(cx: &rustc_lint::EarlyContext<'_>, item: &Item) {
    let ItemKind::Use(use_tree) = &item.kind else {
        return;
    };

    for path_str in use_tree_to_strings(use_tree) {
        for pattern in INFRA_PATTERNS {
            if path_str.starts_with(pattern) {
                cx.span_lint(DE0301_NO_INFRA_IN_DOMAIN, item.span, |diag| {
                    diag.primary_message(
                        format!("domain module imports infrastructure dependency `{}` (DE0301)", pattern)
                    );
                    diag.help("domain should depend only on abstractions; move infrastructure code to infra/ layer");
                });
                return;
            }
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

            for pattern in INFRA_PATTERNS {
                if path_str.starts_with(pattern) {
                    cx.span_lint(DE0301_NO_INFRA_IN_DOMAIN, ty.span, |diag| {
                        diag.primary_message(
                            format!("domain module uses infrastructure type `{}` (DE0301)", path_str)
                        );
                        diag.help("domain should depend only on abstractions; move infrastructure code to infra/ layer");
                    });
                    return;
                }
            }

            // Recursively check generic arguments (e.g., Option<sqlx::PgPool>)
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
        // Handle references: &sqlx::PgPool
        TyKind::Ref(_, mut_ty) => {
            check_type_in_domain(cx, &mut_ty.ty);
        }
        // Handle slices: [sqlx::PgPool]
        TyKind::Slice(inner_ty) => {
            check_type_in_domain(cx, inner_ty);
        }
        // Handle arrays: [sqlx::PgPool; 10]
        TyKind::Array(inner_ty, _) => {
            check_type_in_domain(cx, inner_ty);
        }
        // Handle raw pointers: *const sqlx::PgPool
        TyKind::Ptr(mut_ty) => {
            check_type_in_domain(cx, &mut_ty.ty);
        }
        // Handle tuples: (sqlx::PgPool, String)
        TyKind::Tup(types) => {
            for inner_ty in types {
                check_type_in_domain(cx, inner_ty);
            }
        }
        // Handle trait objects: dyn sqlx::Database
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

                    for pattern in INFRA_PATTERNS {
                        if path_str.starts_with(pattern) {
                            cx.span_lint(DE0301_NO_INFRA_IN_DOMAIN, ty.span, |diag| {
                                diag.primary_message(
                                    format!("domain module uses infrastructure trait `{}` (DE0301)", path_str)
                                );
                                diag.help("domain should depend only on abstractions; move infrastructure code to infra/ layer");
                            });
                            return;
                        }
                    }
                }
            }
        }
        // Handle impl Trait: impl sqlx::Executor
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

                    for pattern in INFRA_PATTERNS {
                        if path_str.starts_with(pattern) {
                            cx.span_lint(DE0301_NO_INFRA_IN_DOMAIN, ty.span, |diag| {
                                diag.primary_message(
                                    format!("domain module uses infrastructure trait `{}` (DE0301)", path_str)
                                );
                                diag.help("domain should depend only on abstractions; move infrastructure code to infra/ layer");
                            });
                            return;
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

impl EarlyLintPass for De0301NoInfraInDomain {
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
        lint_utils::test_comment_annotations_match_stderr(&ui_dir, "DE0301", "infra in domain");
    }
}
