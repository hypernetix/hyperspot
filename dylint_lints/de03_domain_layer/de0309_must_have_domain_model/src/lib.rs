#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;

use lint_utils::is_in_domain_path;
use rustc_ast::{Item, ItemKind};
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};

dylint_linting::declare_pre_expansion_lint! {
    /// DE0309: Domain Structs Must Have `#[domain_model]` Attribute
    ///
    /// All struct and enum types in the domain layer MUST have the `#[domain_model]`
    /// attribute to ensure compile-time validation of DDD boundaries.
    ///
    /// ### Why is this important?
    ///
    /// The `#[domain_model]` macro enforces that domain types don't contain
    /// infrastructure dependencies (HTTP types, database types, etc.) at compile time.
    /// This provides stronger guarantees than import-based lints and prevents
    /// infrastructure leakage into the domain layer.
    ///
    /// ### Example: Bad
    ///
    /// ```rust,ignore
    /// // src/domain/user.rs
    /// pub struct User {           // Missing #[domain_model]
    ///     pub id: Uuid,
    ///     pub email: String,
    /// }
    /// ```
    ///
    /// ### Example: Good
    ///
    /// ```rust,ignore
    /// // src/domain/user.rs
    /// use modkit_macros::domain_model;
    ///
    /// #[domain_model]
    /// pub struct User {
    ///     pub id: Uuid,
    ///     pub email: String,
    /// }
    /// ```
    pub DE0309_MUST_HAVE_DOMAIN_MODEL,
    Deny,
    "domain types must have #[domain_model] attribute for DDD boundary enforcement (DE0309)"
}

impl EarlyLintPass for De0309MustHaveDomainModel {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        check_domain_model_attribute(cx, item);
    }
}

fn check_domain_model_attribute(cx: &EarlyContext<'_>, item: &Item) {
    // Only check structs and enums
    if !matches!(item.kind, ItemKind::Struct(..) | ItemKind::Enum(..)) {
        return;
    }

    // Only check items in domain path
    if !is_in_domain_path(cx.sess().source_map(), item.span) {
        return;
    }

    // Check if the item has #[domain_model] attribute
    if has_domain_model_attribute(item) {
        return;
    }

    // Get item kind and name for error message
    let (item_keyword, item_name) = match &item.kind {
        ItemKind::Struct(ident, ..) => ("struct", ident.name.as_str()),
        ItemKind::Enum(ident, ..) => ("enum", ident.name.as_str()),
        _ => return,
    };

    cx.span_lint(DE0309_MUST_HAVE_DOMAIN_MODEL, item.span, |diag| {
        diag.primary_message(format!(
            "domain type `{item_name}` is missing required #[domain_model] attribute (DE0309)"
        ));
        diag.help(format!(
            "add #[domain_model] attribute to enforce DDD boundaries at compile time: \
             use modkit_macros::domain_model; #[domain_model] pub {item_keyword} ..."
        ));
    });
}

/// Check if an item has the `#[domain_model]` or `#[modkit::domain_model]` attribute.
fn has_domain_model_attribute(item: &Item) -> bool {
    for attr in &item.attrs {
        if let rustc_ast::AttrKind::Normal(attr_item) = &attr.kind {
            let path = &attr_item.item.path;
            let segments: Vec<&str> = path
                .segments
                .iter()
                .map(|s| s.ident.name.as_str())
                .collect();

            // Match: domain_model, modkit::domain_model, modkit_macros::domain_model
            if segments.last() == Some(&"domain_model") {
                return true;
            }
        }
    }
    false
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
            "DE0309",
            "domain_model attribute",
        );
    }
}
