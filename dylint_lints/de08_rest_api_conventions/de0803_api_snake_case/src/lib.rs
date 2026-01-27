#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;
extern crate rustc_span;

use rustc_ast::{Attribute, FieldDef, Item, ItemKind, VariantData};
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
    /// Checks structs and enums in api/rest folders for snake_case compliance.
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
                    check_variant_rename(cx, &variant.attrs);
                    check_fields(cx, &variant.data);
                }
            }
            _ => {}
        }
    }
}

/// Extracts values from serde attributes matching the given name.
///
/// Handles both direct forms like `rename = "value"` and nested forms like
/// `rename(serialize = "value1", deserialize = "value2")`.
///
/// Returns spans and string values for all matching attributes.
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

            // Try to get direct value: rename = "value"
            if let Some(value) = meta_item.value_str() {
                results.push((meta_item.span, value.as_str().to_string()));
            }
            
            // Try to get nested list values: rename(serialize = "value1", deserialize = "value2")
            if let Some(inner_list) = meta_item.meta_item_list() {
                for inner_nested in inner_list {
                    let Some(inner_meta_item) = inner_nested.meta_item() else {
                        continue;
                    };
                    
                    if let Some(inner_value) = inner_meta_item.value_str() {
                        results.push((inner_meta_item.span, inner_value.as_str().to_string()));
                    }
                }
            }
        }
    }
    
    results
}

/// Validates that `rename_all` attributes use the literal "snake_case" value.
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

/// Validates that enum variant `rename` attributes use snake_case values.
fn check_variant_rename(cx: &EarlyContext<'_>, attrs: &[Attribute]) {
    for (span, value) in find_serde_attribute_value(attrs, "rename") {
        if !is_snake_case(&value) {
            cx.span_lint(
                DE0803_API_SNAKE_CASE,
                span,
                |diag| {
                    diag.primary_message(
                        "Enum variants must not use non-snake_case in serde rename (DE0803)"
                    );
                    diag.help("Enum variants in api/rest must use snake_case to match API standards");
                },
            );
        }
    }
}

/// Validates that fields use snake_case names or have a serde rename to snake_case.
fn check_fields(cx: &EarlyContext<'_>, variant_data: &VariantData) {    
    for field in variant_data.fields() {
        check_field_snake_case(cx, field);
    }
}

/// Checks a single field for snake_case compliance.
/// 
/// A field is valid if:
/// 1. The field name is snake_case, OR
/// 2. The field has a `#[serde(rename = "snake_case_value")]` attribute
///
/// A field is invalid if:
/// 1. The field name is not snake_case AND has no serde rename, OR
/// 2. The field has a serde rename to a non-snake_case value
fn check_field_snake_case(cx: &EarlyContext<'_>, field: &FieldDef) {
    let field_name = match &field.ident {
        Some(ident) => ident.name.as_str().to_string(),
        None => return, // Tuple struct fields have no name
    };
    
    let rename_values = find_serde_attribute_value(&field.attrs, "rename");
    
    if rename_values.is_empty() {
        // No field-level serde rename - field name must be snake_case
        if !is_snake_case(&field_name) {
            cx.span_lint(
                DE0803_API_SNAKE_CASE,
                field.ident.unwrap().span,
                |diag| {
                    diag.primary_message(
                        "DTO field name must be snake_case or have a serde rename to snake_case (DE0803)"
                    );
                    diag.help(format!(
                        "rename field to snake_case or add #[serde(rename = \"{}\")]",
                        to_snake_case(&field_name)
                    ));
                },
            );
        }
    } else {
        // Has field-level serde rename - the rename value must be snake_case
        for (span, value) in rename_values {
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

/// Checks if a string is valid snake_case.
/// 
/// Snake case: lowercase letters, digits, and underscores only.
/// Examples: "my_field", "user_id", "field_123"
fn is_snake_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    
    // Must not start or end with underscore
    if s.starts_with('_') || s.ends_with('_') {
        return false;
    }
    
    // Must not have consecutive underscores
    if s.contains("__") {
        return false;
    }
    
    // All characters must be lowercase, digits, or underscore
    s.chars().all(|c| c.is_lowercase() || c.is_ascii_digit() || c == '_')
}

/// Converts a string to snake_case.
fn to_snake_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    result
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
