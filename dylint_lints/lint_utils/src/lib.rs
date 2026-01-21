#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_session;
extern crate rustc_span;

use rustc_span::source_map::SourceMap;
use rustc_span::{FileName, Span};

pub fn is_in_domain_path(source_map: &SourceMap, span: Span) -> bool {
    check_span_path(source_map, span, "/domain/")
}

pub fn is_in_infra_path(source_map: &SourceMap, span: Span) -> bool {
    check_span_path(source_map, span, "/infra/")
}

pub fn is_in_contract_path(source_map: &SourceMap, span: Span) -> bool {
    check_span_path(source_map, span, "/contract/")
}

pub fn is_in_api_rest_folder(source_map: &SourceMap, span: Span) -> bool {
    check_span_path(source_map, span, "/api/rest/")
}

pub fn is_in_module_folder(source_map: &SourceMap, span: Span) -> bool {
    check_span_path(source_map, span, "/modules/")
}

pub fn check_derive_attrs<F>(item: &rustc_ast::Item, mut f: F)
where
    F: FnMut(&rustc_ast::MetaItem, &rustc_ast::Attribute),
{
    for attr in &item.attrs {
        if !attr.has_name(rustc_span::symbol::sym::derive) {
            continue;
        }

        // Parse the derive attribute meta list
        if let rustc_ast::AttrKind::Normal(attr_item) = &attr.kind
            && let Some(meta_items) = attr_item.item.meta_item_list()
        {
            for nested_meta in meta_items {
                if let Some(meta_item) = nested_meta.meta_item() {
                    f(meta_item, attr)
                }
            }
        }
    }
}

pub fn get_derive_path_segments(meta_item: &rustc_ast::MetaItem) -> Vec<&str> {
    let path = &meta_item.path;
    path.segments
        .iter()
        .map(|s| s.ident.name.as_str())
        .collect()
}

/// Check if path segments represent a serde trait (Serialize or Deserialize)
///
/// Handles various forms:
/// - Bare: `Serialize`, `Deserialize`
/// - Qualified: `serde::Serialize`, `serde::Deserialize`
/// - Fully qualified: `::serde::Serialize`
/// ```
pub fn is_serde_trait(segments: &[&str], trait_name: &str) -> bool {
    if segments.is_empty() {
        return false;
    }

    if segments.last() != Some(&trait_name) {
        return false;
    }

    // If it's a qualified path, ensure it contains "serde"
    // Accept: serde::Serialize, ::serde::Serialize
    // Reject: other_crate::Serialize
    if segments.len() >= 2 {
        segments.contains(&"serde")
    } else {
        // Bare identifier: Serialize or Deserialize
        // We accept this as it's commonly used with `use serde::{Serialize, Deserialize}`
        true
    }
}

/// Check if an item has the `#[modkit_macros::api_dto(...)]` attribute.
///
/// The `api_dto` macro automatically adds:
/// - `#[derive(serde::Serialize)]` (if `response` is specified)
/// - `#[derive(serde::Deserialize)]` (if `request` is specified)
/// - `#[derive(utoipa::ToSchema)]` (always)
/// - `#[serde(rename_all = "snake_case")]` (always)
///
/// Lints checking for these derives/attributes should skip items with this attribute.
pub fn has_api_dto_attribute(item: &rustc_ast::Item) -> bool {
    for attr in &item.attrs {
        // Check for modkit_macros::api_dto or just api_dto
        if let rustc_ast::AttrKind::Normal(attr_item) = &attr.kind {
            let path = &attr_item.item.path;
            let segments: Vec<&str> = path.segments.iter().map(|s| s.ident.name.as_str()).collect();

            // Match: api_dto, modkit_macros::api_dto
            if segments.last() == Some(&"api_dto") {
                return true;
            }
        }
    }
    false
}

/// Returns the api_dto arguments (request, response) if present.
/// Returns None if the attribute is not present.
/// Returns Some with flags indicating which modes are enabled.
pub fn get_api_dto_args(item: &rustc_ast::Item) -> Option<ApiDtoArgs> {
    for attr in &item.attrs {
        if let rustc_ast::AttrKind::Normal(attr_item) = &attr.kind {
            let path = &attr_item.item.path;
            let segments: Vec<&str> = path.segments.iter().map(|s| s.ident.name.as_str()).collect();

            if segments.last() != Some(&"api_dto") {
                continue;
            }

            // Parse the arguments
            let mut has_request = false;
            let mut has_response = false;

            if let Some(args) = attr_item.item.meta_item_list() {
                for arg in args {
                    if let Some(ident) = arg.ident() {
                        match ident.name.as_str() {
                            "request" => has_request = true,
                            "response" => has_response = true,
                            _ => {}
                        }
                    }
                }
            }

            return Some(ApiDtoArgs {
                has_request,
                has_response,
            });
        }
    }
    None
}

/// Arguments parsed from `#[api_dto(request, response)]`
#[derive(Debug, Clone, Copy)]
pub struct ApiDtoArgs {
    pub has_request: bool,
    pub has_response: bool,
}

impl ApiDtoArgs {
    /// Returns true if the macro will add Serialize derive (response mode)
    pub fn adds_serialize(&self) -> bool {
        self.has_response
    }

    /// Returns true if the macro will add Deserialize derive (request mode)
    pub fn adds_deserialize(&self) -> bool {
        self.has_request
    }

    /// Returns true if the macro will add ToSchema derive (always)
    pub fn adds_toschema(&self) -> bool {
        true
    }

    /// Returns true if the macro will add serde(rename_all = "snake_case") (always)
    pub fn adds_snake_case_rename(&self) -> bool {
        true
    }
}

// Check if path segments represent a utoipa trait
// Examples: ["ToSchema"], ["utoipa", "ToSchema"], ["utoipa", "ToSchema"]
pub fn is_utoipa_trait(segments: &[&str], trait_name: &str) -> bool {
    if segments.is_empty() {
        return false;
    }

    if segments.last() != Some(&trait_name) {
        return false;
    }

    // If it's a qualified path, ensure it contains "utoipa"
    // Accept: utoipa::ToSchema, ::utoipa::ToSchema
    // Reject: other_crate::ToSchema
    if segments.len() >= 2 {
        segments.contains(&"utoipa")
    } else {
        // Bare identifier: ToSchema
        // We accept this as it's commonly used with `use utoipa::ToSchema`
        true
    }
}

fn check_span_path(source_map: &SourceMap, span: Span, pattern: &str) -> bool {
    let pattern_windows = pattern.replace('/', "\\");
    let path_str =
        get_path_str_from_session(source_map, span).expect("Failed to get test file path");

    // Check for simulated directory in test files first
    if let Some(simulated) = extract_simulated_dir(&path_str) {
        return simulated.contains(pattern) || simulated.contains(&pattern_windows);
    }

    path_str.contains(pattern) || path_str.contains(&pattern_windows)
}

fn get_path_str_from_session(source_map: &SourceMap, span: Span) -> Option<String> {
    let file_name = source_map.span_to_filename(span);

    match file_name {
        FileName::Real(ref real_name) => {
            if let Some(local) = real_name.local_path() {
                return Some(local.to_string_lossy().to_string());
            } else {
                return None;
            }
        }
        _ => return None,
    };
}

/// Extract simulated directory path from a comment at the start of a file.
/// Looks for a comment like: `// simulated_dir=/hyperspot/modules/some_module/contract/`
/// Returns None if no such comment is found.
///
/// Only checks files in temporary directories to avoid unnecessary file I/O in production.
fn extract_simulated_dir(path_str: &str) -> Option<String> {
    // Only check for simulated_dir in temporary paths (tests run in temp directories)
    let is_temp = path_str.contains("/tmp/") 
        || path_str.contains("/var/folders/")  // macOS temp
        || path_str.contains("\\Temp\\")        // Windows temp
        || path_str.contains(".tmp"); // dylint test temp dirs

    if !is_temp {
        return None;
    }

    // Read the first few lines of the file to check for simulated_dir comment
    let contents = std::fs::read_to_string(std::path::PathBuf::from(path_str)).ok()?;

    for line in contents.lines().take(1) {
        let trimmed = line.trim();
        if trimmed.starts_with("// simulated_dir=") {
            return Some(trimmed.trim_start_matches("// simulated_dir=").to_string());
        }
        if !trimmed.is_empty() && !trimmed.starts_with("//") && !trimmed.starts_with("#!") {
            break;
        }
    }

    None
}

/// Test helper function to validate that comment annotations in UI test files match the stderr outputs.
///
/// This function scans all `.rs` files in the specified UI test directory and verifies that:
/// - Lines with a "Should trigger" comment have corresponding errors in the `.stderr` file
/// - Lines with a "Should not trigger" comment do NOT have errors in the `.stderr` file
/// - All errors in `.stderr` files are properly annotated with "Should trigger" comments
///
/// # Arguments
/// * `ui_dir` - Path to the directory containing UI test files
/// * `lint_code` - The lint code to check for in comments (e.g., "DE0101")
/// * `comment_pattern` - The pattern to match in comments (e.g., "Serde in contract")
pub fn test_comment_annotations_match_stderr(
    ui_dir: &std::path::Path,
    lint_code: &str,
    comment_pattern: &str,
) {
    use std::collections::HashSet;
    use std::fs;

    let trigger_comment = format!("// Should trigger {} - {}", lint_code, comment_pattern);
    let not_trigger_comment = format!("// Should not trigger {} - {}", lint_code, comment_pattern);

    // Find all .rs files in ui directory
    let rs_files: Vec<_> = fs::read_dir(ui_dir)
        .expect("Failed to read ui directory")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()? == "rs" {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    assert!(!rs_files.is_empty(), "No .rs test files found in ui directory");

    for rs_file in rs_files {
        let stderr_file = rs_file.with_extension("stderr");

        // Read the .rs file
        let rs_content = fs::read_to_string(&rs_file)
            .unwrap_or_else(|_| panic!("Failed to read {:?}", rs_file));

        // Read the .stderr file (if it exists)
        let stderr_content = fs::read_to_string(&stderr_file).unwrap_or_default();

        // Parse lines from .rs file
        let rs_lines: Vec<&str> = rs_content.lines().collect();

        // Find all lines with "Should trigger" or "Should not trigger" comments
        let mut should_trigger_lines = HashSet::new();
        let mut should_not_trigger_lines = HashSet::new();

        for (idx, line) in rs_lines.iter().enumerate() {
            if line.contains(&trigger_comment) {
                // The next line should have an error (idx + 1 is the next line, +1 again for 1-indexed)
                should_trigger_lines.insert(idx + 2);
            } else if line.contains(&not_trigger_comment) {
                // The next line should NOT have an error
                should_not_trigger_lines.insert(idx + 2);
            }
        }

        // Parse stderr file to find which lines have errors
        let mut error_lines = HashSet::new();
        for line in stderr_content.lines() {
            // Look for lines like "  --> $DIR/file.rs:5:1"
            if line.contains("-->") && line.contains(".rs:") {
                if let Some(pos) = line.rfind(".rs:") {
                    let rest = &line[pos + 4..];
                    if let Some(colon_pos) = rest.find(':') {
                        if let Ok(line_num) = rest[..colon_pos].parse::<usize>() {
                            error_lines.insert(line_num);
                        }
                    }
                }
            }
        }

        // Validate that should_trigger_lines match error_lines
        for line_num in &should_trigger_lines {
            assert!(
                error_lines.contains(line_num),
                "In {:?}: Line {} has '{}' comment but no corresponding error in .stderr file",
                rs_file.file_name().unwrap(),
                line_num,
                trigger_comment
            );
        }

        // Validate that should_not_trigger_lines do NOT appear in error_lines
        for line_num in &should_not_trigger_lines {
            assert!(
                !error_lines.contains(line_num),
                "In {:?}: Line {} has '{}' comment but has an error in .stderr file",
                rs_file.file_name().unwrap(),
                line_num,
                not_trigger_comment
            );
        }

        // Also verify that all error_lines are marked with should_trigger comments
        for line_num in &error_lines {
            assert!(
                should_trigger_lines.contains(line_num),
                "In {:?}: Line {} has an error in .stderr file but no '{}' comment",
                rs_file.file_name().unwrap(),
                line_num,
                trigger_comment
            );
        }
    }
}
