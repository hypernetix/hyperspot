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
        // Check if this is a module named "contract" - recursively check its items - used for test cases
        if let ItemKind::Mod(_, ident, mod_kind) = &item.kind
            // Check if the module is named "contract"
            && ident.name.as_str() == "contract"
        {
            // This is a contract module, check all items within it
            if let rustc_ast::ModKind::Loaded(items, ..) = mod_kind {
                for inner_item in items {
                    check_item_in_contract(cx, inner_item);
                }
            }
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

// Helper to recursively check items within a contract module
fn check_item_in_contract(cx: &EarlyContext<'_>, item: &Item) {
    // Only check structs and enums
    if !matches!(item.kind, ItemKind::Struct(..) | ItemKind::Enum(..)) {
        return;
    }

    check_serde_derives(cx, item);
}

// Helper to check for serde derives on an item
fn check_serde_derives(cx: &EarlyContext<'_>, item: &Item) {
    // Check each attribute for derive with Serialize or Deserialize
    for attr in &item.attrs {
        if !attr.has_name(rustc_span::symbol::sym::derive) {
            continue;
        }

        // Parse the derive attribute meta list
        if let Some(meta_items) = attr.meta_item_list() {
            for meta_item in meta_items {
                if let Some(ident) = meta_item.ident() {
                    let derive_name = ident.name.as_str();

                    if derive_name == "Serialize" {
                        cx.span_lint(DE0101_NO_SERDE_IN_CONTRACT, attr.span, |diag| {
                            diag.primary_message(
                                "contract type should not derive `Serialize` (DE0101)"
                            );
                            diag.help("remove serde derives from contract models; use DTOs in the API layer");
                        });
                    } else if derive_name == "Deserialize" {
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

#[test]
fn ui_examples() {
    dylint_testing::ui_test_examples(env!("CARGO_PKG_NAME"));
}
