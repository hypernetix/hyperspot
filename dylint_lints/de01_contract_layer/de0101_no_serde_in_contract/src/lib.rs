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
        // Only check structs and enums
        if !matches!(item.kind, ItemKind::Struct(..) | ItemKind::Enum(..)) {
            return;
        }

        // Check if we're in a contract module (supports simulated_dir for tests)
        if !is_in_contract_module_ast(cx, item) {
            return;
        }

        // Check for serde derives
        check_serde_derives(cx, item);
    }
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
        if let AttrKind::Normal(attr_item) = &attr.kind
            && let Some(meta_items) = attr_item.item.meta_item_list() {
            for nested_meta in meta_items {
                if let Some(meta_item) = nested_meta.meta_item() {
                    let path = &meta_item.path;
                    let segments: Vec<_> = path.segments.iter()
                        .map(|s| s.ident.name.as_str())
                        .collect();
                    
                    // Check if this is a serde Serialize or Deserialize
                    // Handles: Serialize, serde::Serialize, ::serde::Serialize
                    let is_serialize = lint_utils::is_serde_trait(&segments, "Serialize");
                    let is_deserialize = lint_utils::is_serde_trait(&segments, "Deserialize");

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

#[test]
fn ui_examples() {
    dylint_testing::ui_test_examples(env!("CARGO_PKG_NAME"));
}
