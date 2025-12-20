#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;
extern crate rustc_span;

use rustc_ast::{Item, ItemKind};
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};

use lint_utils::{for_each_item_in_contract_module, is_in_contract_module_ast};

dylint_linting::declare_pre_expansion_lint! {
    /// ### What it does
    ///
    /// Checks that structs and enums in contract modules do not derive Serialize or Deserialize.
    ///
    /// ### Why is this bad?
    ///
    /// Contract models should remain independent of serialization concerns.
    /// Use DTOs (Data Transfer Objects) in the API layer for serialization instead.
    ///
    /// ### Example
    ///
    /// ```rust
    /// // Bad - contract model derives serde traits
    /// mod contract {
    ///     use serde::Serialize;
    ///     #[derive(Serialize)]
    ///     pub struct User { pub id: String }
    /// }
    /// ```
    ///
    /// Use instead:
    ///
    /// ```rust
    /// // Good - contract model without serde
    /// mod contract {
    ///     pub struct User { pub id: String }
    /// }
    ///
    /// // Separate DTO in API layer
    /// mod api {
    ///     use serde::Serialize;
    ///     #[derive(Serialize)]
    ///     pub struct UserDto { pub id: String }
    /// }
    /// ```
    pub DE0101_NO_SERDE_IN_CONTRACT,
    Deny,
    "contract models should not have serde derives (DE0101)"
}

impl EarlyLintPass for De0101NoSerdeInContract {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        // Check if this is an inline "mod contract { ... }" and process items within
        if for_each_item_in_contract_module(cx, item, check_item_in_contract) {
            return;
        }

        // Only check structs and enums
        if !matches!(item.kind, ItemKind::Struct(..) | ItemKind::Enum(..)) {
            return;
        }

        // Check if we're in a contract module by file path (for file-based modules)
        if !is_in_contract_module_ast(cx, item) {
            return;
        }

        // Item is in a file-based contract module, check for serde derives
        check_serde_derives(cx, item);
    }
}

// Helper to check items within a contract module
fn check_item_in_contract(cx: &EarlyContext<'_>, item: &Item) {
    // Only check structs and enums
    if !matches!(item.kind, ItemKind::Struct(..) | ItemKind::Enum(..)) {
        return;
    }

    check_serde_derives(cx, item);
}

// Helper to check for serde derives on an item
fn check_serde_derives(cx: &EarlyContext<'_>, item: &Item) {
    use rustc_ast::ast::AttrKind;
    
    // Check each attribute for derive with Serialize or Deserialize
    for attr in &item.attrs {
        if !attr.has_name(rustc_span::symbol::sym::derive) {
            continue;
        }

        // Parse the derive attribute meta list
        if let AttrKind::Normal(attr_item) = &attr.kind {
            if let Some(meta_items) = attr_item.item.meta_item_list() {
                for nested_meta in meta_items {
                    if let Some(meta_item) = nested_meta.meta_item() {
                        let path = &meta_item.path;
                        let segments: Vec<_> = path.segments.iter()
                            .map(|s| s.ident.name.as_str())
                            .collect();
                        
                        // Check if this is a serde Serialize or Deserialize
                        // Handles: Serialize, serde::Serialize, ::serde::Serialize
                        let is_serialize = is_serde_trait(&segments, "Serialize");
                        let is_deserialize = is_serde_trait(&segments, "Deserialize");

                        if is_serialize {
                            cx.span_lint(DE0101_NO_SERDE_IN_CONTRACT, attr.span, |diag| {
                                diag.primary_message(
                                    "contract type should not derive `Serialize` (DE0101)"
                                );
                                diag.help("remove serde derives from contract models; use DTOs in the API layer");
                            });
                        } else if is_deserialize {
                            cx.span_lint(DE0101_NO_SERDE_IN_CONTRACT, attr.span, |diag| {
                                diag.primary_message(
                                    "contract type should not derive `Deserialize` (DE0101)"
                                );
                                diag.help("remove serde derives from contract models; use DTOs in the API layer");
                            });
                        }
                    }
                }
            }
        }
    }
}

// Check if path segments represent a serde trait
// Examples: ["Serialize"], ["serde", "Serialize"], ["serde", "Serialize"]
fn is_serde_trait(segments: &[&str], trait_name: &str) -> bool {
    if segments.is_empty() {
        return false;
    }
    
    // Check if last segment matches the trait name
    if segments.last() != Some(&trait_name) {
        return false;
    }
    
    // If it's a qualified path, ensure it contains "serde"
    // Accept: serde::Serialize, ::serde::Serialize
    // Reject: other_crate::Serialize
    if segments.len() >= 2 {
        segments.iter().any(|&s| s == "serde")
    } else {
        // Bare identifier: Serialize or Deserialize
        // We accept this as it's commonly used with `use serde::{Serialize, Deserialize}`
        true
    }
}

#[test]
fn ui_examples() {
    dylint_testing::ui_test_examples(env!("CARGO_PKG_NAME"));
}
