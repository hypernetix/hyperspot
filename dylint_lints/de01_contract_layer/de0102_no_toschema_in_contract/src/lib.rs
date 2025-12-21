#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;
extern crate rustc_span;

use rustc_ast::{Item, ItemKind};
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};

use lint_utils::is_in_contract_module_ast;

dylint_linting::declare_pre_expansion_lint! {
    /// ### What it does
    ///
    /// Checks that structs and enums in contract modules do not derive ToSchema.
    ///
    /// ### Why is this bad?
    ///
    /// Contract models should remain independent of OpenAPI documentation concerns.
    /// ToSchema is for API documentation and should only be used on DTOs in the API layer.
    ///
    /// ### Example
    ///
    /// ```rust
    /// // Bad - contract model derives ToSchema
    /// mod contract {
    ///     use utoipa::ToSchema;
    ///     #[derive(ToSchema)]
    ///     pub struct Product { pub id: String }
    /// }
    /// ```
    ///
    /// Use instead:
    ///
    /// ```rust
    /// // Good - contract model without ToSchema
    /// mod contract {
    ///     pub struct Product { pub id: String }
    /// }
    /// 
    /// // Separate DTO in API layer
    /// mod api {
    ///     use utoipa::ToSchema;
    ///     use serde::{Serialize, Deserialize};
    ///     #[derive(Serialize, Deserialize, ToSchema)]
    ///     pub struct ProductDto { pub id: String }
    /// }
    /// ```
    pub DE0102_NO_TOSCHEMA_IN_CONTRACT,
    Deny,
    "contract models should not have ToSchema derive (DE0102)"
}

impl EarlyLintPass for De0102NoToschemaInContract {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        // Only check structs and enums
        if !matches!(item.kind, ItemKind::Struct(..) | ItemKind::Enum(..)) {
            return;
        }

        // Check if we're in a contract module (supports simulated_dir for tests)
        if !is_in_contract_module_ast(cx, item) {
            return;
        }
        
        // Check for ToSchema derives
        check_toschema_derives(cx, item);
    }
}

// Helper to check for ToSchema derives on an item
fn check_toschema_derives(cx: &EarlyContext<'_>, item: &Item) {
    use rustc_ast::ast::AttrKind;
    
    // Check each attribute for derive with ToSchema
    for attr in &item.attrs {
        if !attr.has_name(rustc_span::symbol::sym::derive) {
            continue;
        }

        // Parse the derive attribute meta list
        if let AttrKind::Normal(attr_item) = &attr.kind
            && let Some(meta_items) = attr_item.item.meta_item_list() {
            for nested_meta in meta_items {
                if let Some(meta_item) = nested_meta.meta_item() {
                    let path = &meta_item.path;
                    let segments: Vec<_> = path.segments.iter()
                        .map(|s| s.ident.name.as_str())
                        .collect();
                    
                    // Check if this is a utoipa ToSchema
                    // Handles: ToSchema, utoipa::ToSchema, ::utoipa::ToSchema
                    if is_utoipa_trait(&segments, "ToSchema") {
                        cx.span_lint(DE0102_NO_TOSCHEMA_IN_CONTRACT, attr.span, |diag| {
                            diag.primary_message(
                                "contract type should not derive `ToSchema` (DE0102)"
                            );
                            diag.help("ToSchema is an OpenAPI concern; use DTOs in api/rest/ instead");
                        });
                    }
                }
            }
        }
    }
}

// Check if path segments represent a utoipa trait
// Examples: ["ToSchema"], ["utoipa", "ToSchema"], ["utoipa", "ToSchema"]
fn is_utoipa_trait(segments: &[&str], trait_name: &str) -> bool {
    if segments.is_empty() {
        return false;
    }
    
    // Check if last segment matches the trait name
    if segments.last() != Some(&trait_name) {
        return false;
    }
    
    // If it's a qualified path, ensure it contains "utoipa"
    // Accept: utoipa::ToSchema, ::utoipa::ToSchema
    // Reject: other_crate::ToSchema
    if segments.len() >= 2 {
        segments.contains(&"utoipa")
    } else {
        // Bare identifier: ToSchema
        // We accept this as it's commonly used with `use utoipa::ToSchema`
        true
    }
}

#[test]
fn ui_examples() {
    dylint_testing::ui_test_examples(env!("CARGO_PKG_NAME"));
}
