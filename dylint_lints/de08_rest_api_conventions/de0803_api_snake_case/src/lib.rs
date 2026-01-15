#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;
extern crate rustc_span;

use rustc_ast::{Attribute, Item, ItemKind, VariantData};
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};

use lint_utils::is_in_api_rest_folder;

dylint_linting::declare_pre_expansion_lint! {
    /// DE0803: API DTOs Must Use Snake Case in Serde Attributes
    ///
    /// DTOs must use snake_case in serde rename_all and rename attributes.
    /// This lint checks both:
    /// - Type-level `#[serde(rename_all = "...")]` attributes
    /// - Field-level `#[serde(rename = "...")]` attributes
    ///
    /// Only snake_case is allowed for API consistency per DNA guidelines.
    pub DE0803_API_SNAKE_CASE,
    Deny,
    "API DTOs must use snake_case in serde rename attributes (DE0803)"
}

impl EarlyLintPass for De0803ApiSnakeCase {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        if !is_in_api_rest_folder(cx.sess().source_map(), item.span) {
            return;
        }

        match &item.kind {
            ItemKind::Struct(_, _, variant_data) => {
                check_type_rename_all(cx, &item.attrs);
                check_fields(cx, variant_data);
            }
            ItemKind::Enum(_, _, enum_def) => {
                check_type_rename_all(cx, &item.attrs);
                for variant in &enum_def.variants {
                    check_fields(cx, &variant.data);
                }
            }
            _ => {}
        }
    }
}

fn find_serde_attribute_value(attrs: &[Attribute], attribute_name: &str) -> Vec<(rustc_span::Span, String)> {
    let mut results = Vec::new();
    
    for attr in attrs {
        if !attr.has_name(rustc_span::Symbol::intern("serde")) {
            continue;
        }

        let Some(list) = attr.meta_item_list() else {
            continue;
        };

        for nested in list {
            let Some(meta_item) = nested.meta_item() else {
                continue;
            };

            if !meta_item.has_name(rustc_span::Symbol::intern(attribute_name)) {
                continue;
            }

            let Some(value) = meta_item.value_str() else {
                continue;
            };

            results.push((meta_item.span, value.as_str().to_string()));
        }
    }
    
    results
}

fn check_type_rename_all(cx: &EarlyContext<'_>, attrs: &[Attribute]) {
    for (span, value) in find_serde_attribute_value(attrs, "rename_all") {
        if value != "snake_case" {
            cx.span_lint(
                DE0803_API_SNAKE_CASE,
                span,
                |diag| {
                    diag.primary_message(
                        "DTOs must not use non-snake_case in serde rename_all (DE0803)"
                    );
                    diag.help("DTOs in api/rest must use snake_case (or default) to match API standards");
                },
            );
        }
    }
}

fn check_fields(cx: &EarlyContext<'_>, variant_data: &VariantData) {
    for field in variant_data.fields() {
        for (span, value) in find_serde_attribute_value(&field.attrs, "rename") {
            if !is_snake_case(&value) {
                cx.span_lint(
                    DE0803_API_SNAKE_CASE,
                    span,
                    |diag| {
                        diag.primary_message(
                            "DTO fields must not use non-snake_case in serde rename (DE0803)"
                        );
                        diag.help("DTO fields in api/rest must use snake_case to match API standards");
                    },
                );
            }
        }
    }
}

fn is_snake_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    if s.chars().any(|c| c.is_uppercase()) {
        return false;
    }

    if s.contains('-') || s.contains(' ') {
        return false;
    }

    s.chars().all(|c| c.is_lowercase() || c.is_numeric() || c == '_')
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
            "DE0803", 
            "DTO fields must not use non-snake_case in serde rename/rename_all"
        );
    }
}
