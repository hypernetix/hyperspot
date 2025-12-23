#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;

use rustc_ast::{Item, ItemKind};
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};

use lint_utils::is_in_contract_path;

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

        if !is_in_contract_path(cx.sess().source_map(), item.span) {
            return;
        }

        // Check for serde derives
        lint_utils::check_derive_attrs(item, |meta_item, attr| {
            let segments = lint_utils::get_derive_path_segments(meta_item);

            // Check if this is a serde Serialize or Deserialize
            // Handles: Serialize, serde::Serialize, ::serde::Serialize
            let is_serialize = lint_utils::is_serde_trait(&segments, "Serialize");
            let is_deserialize = lint_utils::is_serde_trait(&segments, "Deserialize");

            if is_serialize {
                cx.span_lint(DE0101_NO_SERDE_IN_CONTRACT, attr.span, |diag| {
                    diag.primary_message("contract type should not derive `Serialize` (DE0101)");
                    diag.help(
                        "remove serde derives from contract models; use DTOs in the API layer",
                    );
                });
            } else if is_deserialize {
                cx.span_lint(DE0101_NO_SERDE_IN_CONTRACT, attr.span, |diag| {
                    diag.primary_message("contract type should not derive `Deserialize` (DE0101)");
                    diag.help(
                        "remove serde derives from contract models; use DTOs in the API layer",
                    );
                });
            }
        });
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
        lint_utils::test_comment_annotations_match_stderr(&ui_dir, "DE0101", "Serde in contract");
    }
}
