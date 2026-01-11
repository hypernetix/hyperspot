#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;

use rustc_ast::{Item, ItemKind, VisibilityKind};
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};

use lint_utils::is_in_contract_path;

dylint_linting::declare_early_lint! {
    /// ### What it does
    ///
    /// Checks that public items in contract modules have documentation comments.
    ///
    /// ### Why is this bad?
    ///
    /// Contract types are the public API of a module and should be well-documented.
    /// Missing documentation makes it hard for consumers to understand the purpose
    /// and usage of exported types.
    ///
    /// ### Example
    ///
    /// ```rust,ignore
    /// // Bad - no doc comment on public struct in contract
    /// pub struct User {
    ///     pub id: Uuid,
    ///     pub name: String,
    /// }
    /// ```
    ///
    /// Use instead:
    ///
    /// ```rust,ignore
    /// /// A user entity representing a registered user in the system.
    /// pub struct User {
    ///     /// Unique identifier for the user.
    ///     pub id: Uuid,
    ///     /// Display name of the user.
    ///     pub name: String,
    /// }
    /// ```
    pub DE1107_PUBLIC_APIS_DOC_COMMENTS,
    Warn,
    "public items in contract modules should have doc comments (DE1107)"
}

fn has_doc_comment(item: &Item) -> bool {
    item.attrs.iter().any(|attr| {
        attr.is_doc_comment()
    })
}

fn is_public(item: &Item) -> bool {
    matches!(item.vis.kind, VisibilityKind::Public)
}

impl EarlyLintPass for De1107PublicApisDocComments {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        // Only check items in contract modules
        if !is_in_contract_path(cx.sess().source_map(), item.span) {
            return;
        }
        
        // Only check public items
        if !is_public(item) {
            return;
        }
        
        // Only check struct, enum, trait, fn, type alias
        let (item_type, item_name) = match &item.kind {
            ItemKind::Struct(ident, ..) => ("struct", ident.name.as_str()),
            ItemKind::Enum(ident, ..) => ("enum", ident.name.as_str()),
            ItemKind::Trait(trt) => ("trait", trt.ident.name.as_str()),
            ItemKind::Fn(f) => ("function", f.ident.name.as_str()),
            ItemKind::TyAlias(t) => ("type alias", t.ident.name.as_str()),
            _ => return,
        };
        
        if !has_doc_comment(item) {
            cx.span_lint(DE1107_PUBLIC_APIS_DOC_COMMENTS, item.span, |diag| {
                diag.primary_message(format!(
                    "public {} `{}` in contract module lacks documentation (DE1107)",
                    item_type,
                    item_name
                ));
                diag.help("add a doc comment (///) explaining the purpose of this item");
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
        lint_utils::test_comment_annotations_match_stderr(&ui_dir, "DE1107", "doc comments");
    }
}
