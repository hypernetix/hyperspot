#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;

use rustc_ast::{Item, ItemKind};
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};

use lint_utils::is_in_api_rest_folder;

dylint_linting::declare_pre_expansion_lint! {
    /// DE0203: DTOs Must Use api_dto Macro
    ///
    /// All DTO types in `api/rest` MUST use the `#[modkit_macros::api_dto(...)]` macro.
    /// The macro ensures consistent serialization behavior by automatically adding
    /// serde derives, ToSchema, and snake_case renaming.
    ///
    /// ### Example: Bad
    ///
    /// ```rust,ignore
    /// // src/api/rest/dto.rs
    /// #[derive(Debug, Clone, Serialize, Deserialize)]  // ❌ Manual derives instead of api_dto
    /// pub struct UserDto {
    ///     pub id: String,
    /// }
    /// ```
    ///
    /// ### Example: Good
    ///
    /// ```rust,ignore
    /// // src/api/rest/dto.rs
    /// #[modkit_macros::api_dto(request, response)]  // ✅ Uses api_dto macro
    /// pub struct UserDto {
    ///     pub id: String,
    /// }
    /// ```
    pub DE0203_DTOS_MUST_USE_API_DTO,
    Deny,
    "DTO types must use the api_dto macro (DE0203)"
}

impl EarlyLintPass for De0203DtosMustUseApiDto {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        check_dto_uses_api_dto(cx, item);
    }
}

fn check_dto_uses_api_dto(cx: &EarlyContext<'_>, item: &Item) {
    // Only check structs and enums
    if !matches!(item.kind, ItemKind::Struct(..) | ItemKind::Enum(..)) {
        return;
    }

    // Only check items in api/rest folder
    if !is_in_api_rest_folder(cx.sess().source_map(), item.span) {
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

    // Check for api_dto macro
    if lint_utils::has_api_dto_attribute(item) {
        return;
    }

    // Report missing api_dto macro
    cx.span_lint(DE0203_DTOS_MUST_USE_API_DTO, item.span, |diag| {
        diag.primary_message("api/rest DTO type must use the api_dto macro (DE0203)");
        diag.help("Use #[modkit_macros::api_dto(request)] for request DTOs, #[modkit_macros::api_dto(response)] for response DTOs, or #[modkit_macros::api_dto(request, response)] for both");
    });
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
        lint_utils::test_comment_annotations_match_stderr(&ui_dir, "DE0203", "DTOs must use api_dto");
    }
}
