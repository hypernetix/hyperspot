#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;

use rustc_ast::{Item, ItemKind, UseTree, UseTreeKind};
use rustc_lint::{EarlyLintPass, LintContext};

use lint_utils::{is_in_modkit_db_path, is_in_hyperspot_server_path};

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

fn starts_with_sqlx(tree: &UseTree) -> bool {
    if let Some(first_seg) = tree.prefix.segments.first() {
        return first_seg.ident.name.as_str() == SQLX_PATTERN;
    }
    false
}

fn check_use_for_sqlx(cx: &rustc_lint::EarlyContext<'_>, item: &Item) {
    let ItemKind::Use(use_tree) = &item.kind else {
        return;
    };

    if starts_with_sqlx(use_tree) {
        let path_str = use_tree_to_string(use_tree);
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
        
        // Check use statements for sqlx imports
        if matches!(item.kind, ItemKind::Use(_)) {
            check_use_for_sqlx(cx, item);
        }
        
        // Check extern crate declarations
        if let ItemKind::ExternCrate(name, _ident) = &item.kind {
            if let Some(sym) = name {
                let sym_str: &str = sym.as_str();
                if sym_str == SQLX_PATTERN {
                    cx.span_lint(DE0706_NO_DIRECT_SQLX, item.span, |diag| {
                        diag.primary_message("extern crate sqlx is prohibited (DE0706)");
                        diag.help("use Sea-ORM EntityTrait or SecORM abstractions instead");
                        diag.note("sqlx bypasses security enforcement and architectural patterns");
                    });
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
            "DE0706",
            "sqlx"
        );
    }
}
