#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;

use rustc_ast::{Item, ItemKind};
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};

use lint_utils::is_in_contract_path;

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

        if !is_in_contract_path(cx.sess().source_map(), item.span) {
            return;
        }

        // Check for ToSchema derives
        lint_utils::check_derive_attrs(item, |meta_item, attr| {
            let segments = lint_utils::get_derive_path_segments(meta_item);

            // Check if this is a utoipa ToSchema
            // Handles: ToSchema, utoipa::ToSchema, ::utoipa::ToSchema
            if lint_utils::is_utoipa_trait(&segments, "ToSchema") {
                cx.span_lint(DE0102_NO_TOSCHEMA_IN_CONTRACT, attr.span, |diag| {
                    diag.primary_message("contract type should not derive `ToSchema` (DE0102)");
                    diag.help("ToSchema is an OpenAPI concern; use DTOs in api/rest/ instead");
                });
            }
        });
    }
}

#[test]
fn ui_examples() {
    dylint_testing::ui_test_examples(env!("CARGO_PKG_NAME"));
}
