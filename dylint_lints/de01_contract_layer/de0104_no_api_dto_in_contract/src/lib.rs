#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;

use rustc_ast::{Item, ItemKind};
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};

use lint_utils::is_in_contract_module_ast;

dylint_linting::declare_pre_expansion_lint! {
    /// ### What it does
    ///
    /// Checks that structs and enums in contract modules do not use the `api_dto` attribute macro.
    ///
    /// ### Why is this bad?
    ///
    /// Contract models should remain independent of API serialization concerns.
    /// The `api_dto` macro is specifically designed for API DTOs (Data Transfer Objects)
    /// and should only be used in the API layer, not in contract models.
    ///
    /// ### Example
    ///
    /// ```rust
    /// // Bad - contract model uses api_dto
    /// mod contract {
    ///     #[modkit_macros::api_dto(request, response)]
    ///     pub struct User { pub id: String }
    /// }
    /// ```
    ///
    /// Use instead:
    ///
    /// ```rust
    /// // Good - contract model without api_dto
    /// mod contract {
    ///     pub struct User { pub id: String }
    /// }
    ///
    /// // Separate DTO in API layer
    /// mod api {
    ///     #[modkit_macros::api_dto(request, response)]
    ///     pub struct UserDto { pub id: String }
    /// }
    /// ```
    pub DE0104_NO_API_DTO_IN_CONTRACT,
    Deny,
    "contract models should not use api_dto macro (DE0104)"
}

impl EarlyLintPass for De0104NoApiDtoInContract {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        // Only check structs and enums
        if !matches!(item.kind, ItemKind::Struct(..) | ItemKind::Enum(..)) {
            return;
        }

        if !is_in_contract_module_ast(cx, item) {
            return;
        }

        // Check for api_dto attribute macro
        for attr in &item.attrs {
            if let rustc_ast::AttrKind::Normal(attr_item) = &attr.kind {
                let path = &attr_item.item.path;
                let segments: Vec<&str> = path.segments.iter().map(|s| s.ident.name.as_str()).collect();
                
                // Check if this is an api_dto attribute
                // Handles: api_dto, modkit_macros::api_dto, ::modkit_macros::api_dto
                let is_api_dto = match segments.as_slice() {
                    ["api_dto"] => true,
                    [.., "modkit_macros", "api_dto"] => true,
                    _ => false,
                };

                if is_api_dto {
                    cx.span_lint(DE0104_NO_API_DTO_IN_CONTRACT, attr.span, |diag| {
                        diag.primary_message("contract type should not use `api_dto` macro (DE0104)");
                        diag.help("api_dto is for API DTOs; use plain structs in contract/ and create DTOs in api/rest/");
                    });
                }
            }
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
        lint_utils::test_comment_annotations_match_stderr(&ui_dir, "DE0104", "api_dto in contract");
    }
}
