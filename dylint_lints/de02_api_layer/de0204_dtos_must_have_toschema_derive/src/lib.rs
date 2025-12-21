#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;
extern crate rustc_span;

use rustc_ast::{Item, ItemKind};
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};

dylint_linting::declare_pre_expansion_lint! {
    /// DE0204: DTOs Must Have ToSchema Derive
    ///
    /// All DTO types MUST derive `utoipa::ToSchema` for OpenAPI documentation.
    /// DTOs in api/rest need schema definitions for API documentation.
    ///
    /// ### Example: Bad
    ///
    /// ```rust,ignore
    /// // src/api/rest/dto.rs
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Debug, Serialize, Deserialize)]  // ❌ Missing ToSchema
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
    /// use utoipa::ToSchema;
    ///
    /// #[derive(Debug, Serialize, Deserialize, ToSchema)]  // ✅ Has ToSchema
    /// pub struct UserDto {
    ///     pub id: String,
    /// }
    /// ```
    pub DE0204_DTOS_MUST_HAVE_TOSCHEMA_DERIVE,
    Deny,
    "DTO types must derive ToSchema for OpenAPI documentation (DE0204)"
}

impl EarlyLintPass for De0204DtosMustHaveToschemaDerive {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        check_dto_toschema_derive(cx, item);
    }
}

fn check_dto_toschema_derive(cx: &EarlyContext<'_>, item: &Item) {
    use rustc_ast::ast::AttrKind;

    // Only check structs and enums
    if !matches!(item.kind, ItemKind::Struct(..) | ItemKind::Enum(..)) {
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

    // Check for ToSchema derive
    let mut has_toschema = false;

    for attr in &item.attrs {
        if !attr.has_name(rustc_span::symbol::sym::derive) {
            continue;
        }

        if let AttrKind::Normal(attr_item) = &attr.kind {
            if let Some(meta_items) = attr_item.item.meta_item_list() {
                for nested_meta in meta_items {
                    if let Some(meta_item) = nested_meta.meta_item() {
                        let path = &meta_item.path;
                        let segments: Vec<_> = path.segments.iter()
                            .map(|s| s.ident.name.as_str())
                            .collect();

                        // Check for ToSchema (bare or utoipa::ToSchema)
                        if is_toschema_trait(&segments) {
                            has_toschema = true;
                        }
                    }
                }
            }
        }
    }

    // Report missing derive
    if !has_toschema {
        cx.span_lint(DE0204_DTOS_MUST_HAVE_TOSCHEMA_DERIVE, item.span, |diag| {
            diag.primary_message(
                "api/rest type is missing required ToSchema derive (DE0204)"
            );
            diag.help("DTOs in api/rest must derive ToSchema for OpenAPI documentation");
        });
    }
}

// Check if path segments represent ToSchema trait
fn is_toschema_trait(segments: &[&str]) -> bool {
    if segments.is_empty() {
        return false;
    }

    // Check if last segment is "ToSchema"
    if segments.last() != Some(&"ToSchema") {
        return false;
    }

    // If qualified path, ensure it contains "utoipa"
    if segments.len() >= 2 {
        segments.contains(&"utoipa")
    } else {
        // Bare identifier accepted (common with use statements)
        true
    }
}

#[test]
fn ui_examples() {
    dylint_testing::ui_test_examples(env!("CARGO_PKG_NAME"));
}
