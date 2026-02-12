#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;

use rustc_ast::ItemKind;
use rustc_lint::{EarlyLintPass, LintContext};

dylint_linting::declare_early_lint! {
    /// DE0201: DTOs Only in API Rest Folder
    ///
    /// Types with DTO suffixes must be defined only in `*/api/rest/*.rs` files.
    pub DE0201_DTOS_ONLY_IN_API_REST,
    Deny,
    "DTO types should only be defined in */api/rest/* files (DE0201)"
}

impl EarlyLintPass for De0201DtosOnlyInApiRest {
    fn check_item(&mut self, cx: &rustc_lint::EarlyContext<'_>, item: &rustc_ast::Item) {
        // Only check structs and enums
        if !matches!(item.kind, ItemKind::Struct(..) | ItemKind::Enum(..)) {
            return;
        }

        // Check if item name ends with "Dto"
        let (item_name, span) = match &item.kind {
            ItemKind::Struct(ident, ..) => {
                let span = item.span.with_hi(ident.span.hi());
                (ident.name.as_str(), span)
            }
            ItemKind::Enum(ident, ..) => {
                let span = item.span.with_hi(ident.span.hi());
                (ident.name.as_str(), span)
            }
            _ => return,
        };

        if !item_name.to_lowercase().ends_with("dto") {
            return;
        }

        // Check if the file is in api/rest folder (supports simulated_dir for tests)
        if !lint_utils::is_in_api_rest_folder(cx.sess().source_map(), item.span) {
            cx.span_lint(DE0201_DTOS_ONLY_IN_API_REST, span, |diag| {
                diag.primary_message(format!(
                    "DTO type `{}` is defined outside of api/rest folder (DE0201)",
                    item_name
                ));
                diag.help("move DTO types to src/api/rest/dto.rs");
            });
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
        lint_utils::test_comment_annotations_match_stderr(
            &ui_dir,
            "DE0201",
            "DTOs only in api/rest",
        );
    }
}
