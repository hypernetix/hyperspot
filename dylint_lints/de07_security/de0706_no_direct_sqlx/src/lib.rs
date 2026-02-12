#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;

use lint_utils::{
    is_in_contract_module_ast, is_in_hyperspot_server_path, is_in_modkit_db_path,
    use_tree_to_strings,
};
use rustc_ast::{Item, ItemKind, Ty, TyKind};
use rustc_lint::{EarlyLintPass, LintContext};

dylint_linting::declare_early_lint! {
    /// ### What it does
    ///
    /// Prohibits direct usage of the `sqlx` crate. Projects should use Sea-ORM
    /// or SecORM abstractions instead for database operations.
    ///
    /// ### Why is this bad?
    ///
    /// Direct sqlx usage bypasses important architectural layers:
    /// - Skips security enforcement (SecureConn, AccessScope)
    /// - Bypasses query building abstractions and type safety
    /// - Makes it harder to maintain consistent patterns across the codebase
    /// - Loses automatic audit logging and tenant isolation
    ///
    /// ### Known Exclusions
    ///
    /// This lint does NOT apply to `libs/modkit-db/` which is the internal
    /// wrapper library that provides the Sea-ORM/SecORM abstraction layer.
    ///
    /// ### Example
    ///
    /// ```rust,ignore
    /// // Bad - direct sqlx usage
    /// use sqlx::PgPool;
    /// sqlx::query("SELECT * FROM users").fetch_all(&pool).await?;
    /// ```
    ///
    /// Use instead:
    ///
    /// ```rust,ignore
    /// // Good - use Sea-ORM with SecureConn
    /// use sea_orm::EntityTrait;
    /// UserEntity::find().secure().scope_with(&scope).all(conn).await?;
    /// ```
    pub DE0706_NO_DIRECT_SQLX,
    Deny,
    "direct sqlx usage is prohibited; use Sea-ORM or SecORM instead (DE0706)"
}

/// Sqlx crate pattern to detect
const SQLX_PATTERN: &str = "sqlx";

/// Check if a path string matches the sqlx crate pattern.
/// Matches "sqlx" exactly or any qualified path starting with "sqlx::" (e.g., "sqlx::PgPool").
fn is_sqlx_path(path: &str) -> bool {
    path == SQLX_PATTERN || path.starts_with("sqlx::")
}

/// Find any sqlx path in the use tree (handles grouped imports like `use {sqlx::PgPool, other};`)
fn find_sqlx_path(tree: &rustc_ast::UseTree) -> Option<String> {
    use_tree_to_strings(tree)
        .into_iter()
        .find(|path| is_sqlx_path(path))
}

/// Recursively check a type AST node for sqlx usage.
/// Handles qualified paths like `sqlx::pool::Pool<sqlx::Any>` in struct fields,
/// function parameters, return types, and type aliases.
fn check_type_for_sqlx(cx: &rustc_lint::EarlyContext<'_>, ty: &Ty) {
    match &ty.kind {
        TyKind::Path(_, path) => {
            let path_str = path
                .segments
                .iter()
                .map(|seg| seg.ident.name.as_str())
                .collect::<Vec<_>>()
                .join("::");

            if is_sqlx_path(&path_str) {
                cx.span_lint(DE0706_NO_DIRECT_SQLX, ty.span, |diag| {
                    diag.primary_message(format!(
                        "direct sqlx type usage detected: `{path_str}` (DE0706)"
                    ));
                    diag.help("use Sea-ORM EntityTrait or SecORM abstractions instead");
                    diag.note("sqlx bypasses security enforcement and architectural patterns");
                });
                return;
            }

            // Recursively check generic arguments (e.g., Option<sqlx::PgPool>)
            for segment in &path.segments {
                if let Some(args) = &segment.args {
                    if let rustc_ast::GenericArgs::AngleBracketed(ref angle_args) = **args {
                        for arg in &angle_args.args {
                            if let rustc_ast::AngleBracketedArg::Arg(
                                rustc_ast::GenericArg::Type(inner_ty),
                            ) = arg
                            {
                                check_type_for_sqlx(cx, inner_ty);
                            }
                        }
                    }
                }
            }
        }
        TyKind::Ref(_, mut_ty) => {
            check_type_for_sqlx(cx, &mut_ty.ty);
        }
        TyKind::Slice(inner_ty) | TyKind::Array(inner_ty, _) => {
            check_type_for_sqlx(cx, inner_ty);
        }
        TyKind::Ptr(mut_ty) => {
            check_type_for_sqlx(cx, &mut_ty.ty);
        }
        TyKind::Tup(types) => {
            for inner_ty in types {
                check_type_for_sqlx(cx, inner_ty);
            }
        }
        TyKind::TraitObject(bounds, _) | TyKind::ImplTrait(_, bounds) => {
            for bound in bounds {
                if let rustc_ast::GenericBound::Trait(trait_ref) = bound {
                    let path = &trait_ref.trait_ref.path;
                    let path_str = path
                        .segments
                        .iter()
                        .map(|seg| seg.ident.name.as_str())
                        .collect::<Vec<_>>()
                        .join("::");

                    if is_sqlx_path(&path_str) {
                        cx.span_lint(DE0706_NO_DIRECT_SQLX, ty.span, |diag| {
                            diag.primary_message(format!(
                                "direct sqlx trait usage detected: `{path_str}` (DE0706)"
                            ));
                            diag.help(
                                "use Sea-ORM EntityTrait or SecORM abstractions instead",
                            );
                            diag.note(
                                "sqlx bypasses security enforcement and architectural patterns",
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

fn check_use_for_sqlx(cx: &rustc_lint::EarlyContext<'_>, item: &Item) {
    let ItemKind::Use(use_tree) = &item.kind else {
        return;
    };

    if let Some(path_str) = find_sqlx_path(use_tree) {
        cx.span_lint(DE0706_NO_DIRECT_SQLX, item.span, |diag| {
            diag.primary_message(format!(
                "direct sqlx import detected: `{}` (DE0706)",
                path_str
            ));
            diag.help("use Sea-ORM EntityTrait or SecORM abstractions instead");
            diag.note("sqlx bypasses security enforcement and architectural patterns");
        });
    }
}

impl EarlyLintPass for De0706NoDirectSqlx {
    fn check_item(&mut self, cx: &rustc_lint::EarlyContext<'_>, item: &Item) {
        // Skip libs/modkit-db/ - this is the internal wrapper library
        // that legitimately uses sqlx to provide the abstraction layer
        if is_in_modkit_db_path(cx.sess().source_map(), item.span) {
            return;
        }

        // Skip apps/hyperspot-server/ - it needs sqlx driver linkage workaround
        if is_in_hyperspot_server_path(cx.sess().source_map(), item.span) {
            return;
        }

        // Skip contract/ modules - they may need sqlx types for test fixtures
        if is_in_contract_module_ast(cx, item) {
            return;
        }

        match &item.kind {
            // Check use statements for sqlx imports
            ItemKind::Use(_) => {
                check_use_for_sqlx(cx, item);
            }
            // Check extern crate declarations
            ItemKind::ExternCrate(rename, ident) => {
                let is_sqlx = match rename {
                    Some(sym) => sym.as_str() == SQLX_PATTERN,
                    None => ident.name.as_str() == SQLX_PATTERN,
                };

                if is_sqlx {
                    cx.span_lint(DE0706_NO_DIRECT_SQLX, item.span, |diag| {
                        diag.primary_message("extern crate sqlx is prohibited (DE0706)");
                        diag.help("use Sea-ORM EntityTrait or SecORM abstractions instead");
                        diag.note(
                            "sqlx bypasses security enforcement and architectural patterns",
                        );
                    });
                }
            }
            // Check struct fields for sqlx types
            ItemKind::Struct(_, _, variant_data) => {
                for field in variant_data.fields() {
                    check_type_for_sqlx(cx, &field.ty);
                }
            }
            // Check enum variant fields for sqlx types
            ItemKind::Enum(_, _, enum_def) => {
                for variant in &enum_def.variants {
                    for field in variant.data.fields() {
                        check_type_for_sqlx(cx, &field.ty);
                    }
                }
            }
            // Check function parameter and return types
            ItemKind::Fn(fn_item) => {
                for param in &fn_item.sig.decl.inputs {
                    check_type_for_sqlx(cx, &param.ty);
                }
                if let rustc_ast::FnRetTy::Ty(ret_ty) = &fn_item.sig.decl.output {
                    check_type_for_sqlx(cx, ret_ty);
                }
            }
            // Check type alias targets
            ItemKind::TyAlias(ty_alias) => {
                if let Some(ty) = &ty_alias.ty {
                    check_type_for_sqlx(cx, ty);
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
        lint_utils::test_comment_annotations_match_stderr(&ui_dir, "DE0706", "sqlx");
    }
}
