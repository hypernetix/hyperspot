#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;
extern crate rustc_span;

use rustc_ast::{Item, ItemKind};
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};

use lint_utils::is_in_api_rest_folder_ast;

dylint_linting::declare_pre_expansion_lint! {
    /// DE0203: DTOs Must Have Serde Derives
    ///
    /// All DTO types MUST derive `Serialize` and `Deserialize`.
    /// DTOs are for serialization; missing derives cause runtime errors.
    ///
    /// ### Example: Bad
    ///
    /// ```rust,ignore
    /// // src/api/rest/dto.rs
    /// #[derive(Debug, Clone)]  // ❌ Missing serde derives
    /// pub struct UserDto {
    ///     pub id: String,
    /// }
    /// ```
    ///
    /// ### Example: Good
    ///
    /// ```rust,ignore
    /// // src/api/rest/dto.rs
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Debug, Clone, Serialize, Deserialize)]  // ✅ Has serde derives
    /// pub struct UserDto {
    ///     pub id: String,
    /// }
    /// ```
    pub DE0203_DTOS_MUST_HAVE_SERDE_DERIVES,
    Deny,
    "DTO types must derive Serialize and Deserialize (DE0203)"
}

impl EarlyLintPass for De0203DtosMustHaveSerdeDerives {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        check_dto_serde_derives(cx, item);
    }
}

fn check_dto_serde_derives(cx: &EarlyContext<'_>, item: &Item) {
    use rustc_ast::ast::AttrKind;

    // Only check structs and enums
    if !matches!(item.kind, ItemKind::Struct(..) | ItemKind::Enum(..)) {
        return;
    }

    // Only check items in api/rest folder
    if !is_in_api_rest_folder_ast(cx, item) {
        return;
    }

    // Check if the type name ends with "Dto" suffix (case-insensitive)
    let item_name = match &item.kind {
        ItemKind::Struct(ident, _, _) => ident.name.as_str(),
        ItemKind::Enum(ident, _, _) => ident.name.as_str(),
        _ => return,
    };
    let item_name_lower = item_name.to_lowercase();
    if !item_name_lower.ends_with("dto") {
        return;
    }

    // Check for Serialize and Deserialize derives
    let mut has_serialize = false;
    let mut has_deserialize = false;

    for attr in &item.attrs {
        if !attr.has_name(rustc_span::symbol::sym::derive) {
            continue;
        }

        if let AttrKind::Normal(attr_item) = &attr.kind
            && let Some(meta_items) = attr_item.item.meta_item_list() {
            for nested_meta in meta_items {
                if let Some(meta_item) = nested_meta.meta_item() {
                    let path = &meta_item.path;
                    let segments: Vec<_> = path.segments.iter()
                        .map(|s| s.ident.name.as_str())
                        .collect();

                    // Check for Serialize (bare or serde::Serialize)
                    if lint_utils::is_serde_trait(&segments, "Serialize") {
                        has_serialize = true;
                    }
                    // Check for Deserialize (bare or serde::Deserialize)
                    if lint_utils::is_serde_trait(&segments, "Deserialize") {
                        has_deserialize = true;
                    }
                }
            }
        }
    }

    // Report missing derives
    if !has_serialize || !has_deserialize {
        let mut missing = Vec::new();
        if !has_serialize {
            missing.push("Serialize");
        }
        if !has_deserialize {
            missing.push("Deserialize");
        }

        cx.span_lint(DE0203_DTOS_MUST_HAVE_SERDE_DERIVES, item.span, |diag| {
            diag.primary_message(format!(
                "api/rest type is missing required serde derives: {} (DE0203)",
                missing.join(", ")
            ));
            diag.help("DTOs in api/rest must derive both Serialize and Deserialize");
        });
    }
}

#[test]
fn ui_examples() {
    dylint_testing::ui_test_examples(env!("CARGO_PKG_NAME"));
}
