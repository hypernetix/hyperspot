#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_session;
extern crate rustc_span;

use rustc_lint::LintContext;
use rustc_span::source_map::SourceMap;
use rustc_span::{FileName, RealFileName, Span};
use std::collections::HashSet;

const ALLOWED_FLAGS: &[&str] = &["request", "response"];

pub fn is_in_domain_path(source_map: &SourceMap, span: Span) -> bool {
    check_span_path(source_map, span, "/domain/")
}

pub fn is_in_infra_path(source_map: &SourceMap, span: Span) -> bool {
    check_span_path(source_map, span, "/infra/")
}

pub fn is_in_contract_path(source_map: &SourceMap, span: Span) -> bool {
    check_span_path(source_map, span, "/contract/")
}

/// AST-based helper to check if an item is in a contract module.
/// This works with EarlyLintPass and checks both file paths and simulated_dir comments.
pub fn is_in_contract_module_ast(cx: &rustc_lint::EarlyContext<'_>, item: &rustc_ast::Item) -> bool {
    is_in_contract_path(cx.sess().source_map(), item.span)
}

pub fn is_in_api_rest_folder(source_map: &SourceMap, span: Span) -> bool {
    check_span_path(source_map, span, "/api/rest/")
}

pub fn is_in_module_folder(source_map: &SourceMap, span: Span) -> bool {
    check_span_path(source_map, span, "/modules/")
}

/// Extract the filename string from a span.
/// Handles local paths and remapped paths with virtual name fallback.
pub fn filename_str(source_map: &SourceMap, span: Span) -> Option<String> {
    let file_name = source_map.span_to_filename(span);
    match &file_name {
        FileName::Real(real) => {
            if let Some(local) = real.local_path() {
                Some(local.to_string_lossy().to_string())
            } else if let RealFileName::Remapped { virtual_name, .. } = real {
                Some(virtual_name.to_string_lossy().to_string())
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Check if a file path is in a temporary directory (used by test infrastructure).
pub fn is_temp_path(path: &str) -> bool {
    // Primary check: compare against the actual system temp directory
    let temp_dir = std::env::temp_dir();
    if let Some(temp_str) = temp_dir.to_str() {
        if path.starts_with(temp_str) {
            return true;
        }
    }
    // Fallback patterns for known temp directory locations
    path.contains("/tmp/")
        || path.contains("/var/folders/")
        || path.contains("\\Temp\\")
}

/// Result of parsing a version suffix from a name like `FooClientV1` or `FooClient2`.
pub struct VersionParts<'a> {
    /// Base name without version suffix or trailing digits (e.g., `FooClient`)
    pub base: &'a str,
    /// Valid version suffix like `V1`, `V2`, or empty string if none
    pub version_suffix: &'a str,
    /// Trailing digits without V prefix (e.g., `2` from `FooClient2`), or empty string
    pub malformed_digits: &'a str,
}

impl VersionParts<'_> {
    /// Returns true if a valid version suffix (V + digits) was found.
    pub fn has_valid_version(&self) -> bool {
        !self.version_suffix.is_empty()
    }

    /// Returns true if there are trailing digits but no V prefix (malformed version).
    pub fn has_malformed_version(&self) -> bool {
        !self.malformed_digits.is_empty() && self.version_suffix.is_empty()
    }
}

/// Parse version suffix from a trait/type name.
///
/// - `FooClientV1`  -> base=`FooClient`, version_suffix=`V1`, malformed_digits=``
/// - `FooClientV10` -> base=`FooClient`, version_suffix=`V10`, malformed_digits=``
/// - `FooClient2`   -> base=`FooClient`, version_suffix=``, malformed_digits=`2`
/// - `FooClient`    -> base=`FooClient`, version_suffix=``, malformed_digits=``
/// - `FooClientV`   -> base=`FooClient`, version_suffix=``, malformed_digits=`` (bare V stripped)
/// - `FooClientV0`  -> base=`FooClient`, version_suffix=``, malformed_digits=`` (V0 rejected)
/// - `FooClientV01` -> base=`FooClient`, version_suffix=``, malformed_digits=`` (leading zero rejected)
pub fn parse_version_suffix(name: &str) -> VersionParts<'_> {
    if name.is_empty() {
        return VersionParts {
            base: name,
            version_suffix: "",
            malformed_digits: "",
        };
    }

    let bytes = name.as_bytes();
    let len = bytes.len();

    let mut digit_count = 0;
    for &b in bytes.iter().rev() {
        if b.is_ascii_digit() {
            digit_count += 1;
        } else {
            break;
        }
    }

    if digit_count == 0 {
        // No trailing digits — check for bare trailing V (e.g., `FooClientV`)
        if len > 1 && bytes[len - 1] == b'V' {
            return VersionParts {
                base: &name[..len - 1],
                version_suffix: "",
                malformed_digits: "",
            };
        }
        return VersionParts {
            base: name,
            version_suffix: "",
            malformed_digits: "",
        };
    }

    let digits_start = len - digit_count;

    if digits_start > 0 && bytes[digits_start - 1] == b'V' {
        let v_pos = digits_start - 1;
        let digit_str = &name[digits_start..];

        // Valid version: V followed by non-zero number without leading zeros (V1, V2, V10)
        // Invalid: V0, V00, V01 (leading zeros or zero version)
        if !digit_str.starts_with('0') {
            VersionParts {
                base: &name[..v_pos],
                version_suffix: &name[v_pos..],
                malformed_digits: "",
            }
        } else {
            // V0, V00, V01 — strip the invalid V-prefix version from base
            VersionParts {
                base: &name[..v_pos],
                version_suffix: "",
                malformed_digits: "",
            }
        }
    } else {
        VersionParts {
            base: &name[..digits_start],
            version_suffix: "",
            malformed_digits: &name[digits_start..],
        }
    }
}

/// Check if the current compilation target is an SDK crate (by crate name or file path).
///
/// Also returns true for files in temporary directories — this is required because
/// `dylint_testing::ui_test_examples()` compiles UI test files from temp dirs without
/// passing `--crate-name`, so the crate name check alone doesn't work for UI tests.
pub fn is_in_sdk_crate(cx: &rustc_lint::EarlyContext<'_>, span: Span) -> bool {
    if let Some(crate_name) = cx.sess().opts.crate_name.as_deref() {
        // Cargo normalizes dashes to underscores for `--crate-name`.
        if crate_name.ends_with("-sdk") || crate_name.ends_with("_sdk") {
            return true;
        }
    }

    let Some(file_path) = filename_str(cx.sess().source_map(), span) else {
        return false;
    };

    file_path.contains("-sdk/")
        || file_path.contains("-sdk\\")
        || is_temp_path(&file_path)
}

/// Check if span is within libs/modkit-db/ - the internal sqlx wrapper library
/// This path is excluded from sqlx restrictions as it provides the abstraction layer
pub fn is_in_modkit_db_path(source_map: &SourceMap, span: Span) -> bool {
    // Multiple checks handle different path contexts:
    // - "/libs/modkit-db/" - absolute path from workspace root
    // - "libs/modkit-db/" - relative path in some contexts
    // - "modkit-db/src/" - simulated_dir paths in tests
    check_span_path(source_map, span, "/libs/modkit-db/")
        || check_span_path(source_map, span, "libs/modkit-db/")
        || check_span_path(source_map, span, "modkit-db/src/")
}

/// Check if span is within apps/hyperspot-server - the main server binary
/// This path is excluded from sqlx restrictions as it needs driver linkage workaround
pub fn is_in_hyperspot_server_path(source_map: &SourceMap, span: Span) -> bool {
    // Multiple checks handle different path contexts:
    // - "/apps/hyperspot-server/" - absolute path from workspace root
    // - "apps/hyperspot-server/" - relative path in some contexts
    // - "hyperspot-server/src/" - simulated_dir paths in tests
    check_span_path(source_map, span, "/apps/hyperspot-server/")
        || check_span_path(source_map, span, "apps/hyperspot-server/")
        || check_span_path(source_map, span, "hyperspot-server/src/")
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
/// - `#[serde(rename_all = "snake_case")]` (if `request` or `response` are specified)
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

/// Returns the api_dto arguments (request, response) if present and valid.
/// Returns None if the attribute is not present OR if it contains invalid flags.
/// Returns Some with flags indicating which modes are enabled.
///
/// # Validation
///
/// This function validates the attribute arguments to match the proc-macro's behavior:
/// - Only "request" and "response" flags are allowed
/// - Duplicate flags are rejected
/// - Unknown flags are rejected
/// - At least one of "request" or "response" must be present
///
/// If any validation fails, this function returns `None`, treating the invalid
/// attribute the same as an absent attribute. This ensures lint behavior stays
/// in sync with the proc-macro, which would reject these attributes at compile time.
pub fn get_api_dto_args(item: &rustc_ast::Item) -> Option<ApiDtoArgs> {
    for attr in &item.attrs {
        if let rustc_ast::AttrKind::Normal(attr_item) = &attr.kind {
            let path = &attr_item.item.path;
            let segments: Vec<&str> = path.segments.iter().map(|s| s.ident.name.as_str()).collect();

            if segments.last() != Some(&"api_dto") {
                continue;
            }

            // Parse and validate the arguments
            let mut has_request = false;
            let mut has_response = false;
            let mut seen_flags = HashSet::new();
            let mut has_invalid = false;

            if let Some(args) = attr_item.item.meta_item_list() {
                for arg in args {
                    if let Some(ident) = arg.ident() {
                        let flag_str = ident.name.as_str();
                        
                        // Check if flag is allowed
                        if !ALLOWED_FLAGS.contains(&flag_str) {
                            has_invalid = true;
                            break;
                        }
                        
                        // Check for duplicates (convert to String for storage)
                        if !seen_flags.insert(flag_str.to_string()) {
                            has_invalid = true;
                            break;
                        }
                        
                        match flag_str {
                            "request" => has_request = true,
                            "response" => has_response = true,
                            _ => unreachable!(),
                        }
                    }
                }
            }

            // Reject invalid attributes by returning None
            if has_invalid {
                return None;
            }
            
            // Reject empty attributes (no request or response)
            if !has_request && !has_response {
                return None;
            }

            return Some(ApiDtoArgs {
                has_request,
                has_response,
            });
        }
    }
    None
}

/// Arguments parsed from a valid `#[api_dto(request, response)]` attribute.
///
/// # Validity
///
/// This struct is only returned by `get_api_dto_args()` for valid attributes.
/// Invalid attributes (unknown flags, duplicates, or empty) cause `get_api_dto_args()`
/// to return `None` instead.
///
/// A valid `api_dto` attribute has:
/// - At least one of `request` or `response`
/// - Only "request" and "response" flags (no unknown flags)
/// - No duplicate flags
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

    /// Returns true if the macro will add ToSchema derive.
    /// Always returns true because `ApiDtoArgs` only exists for valid attributes.
    pub fn adds_toschema(&self) -> bool {
        true
    }

    /// Returns true if the macro will add serde(rename_all = "snake_case").
    /// This is added when serde derives are present (i.e., at least one mode is enabled).
    /// Always returns true for valid `ApiDtoArgs` since validation requires at least one mode.
    pub fn adds_snake_case_rename(&self) -> bool {
        // Matches proc-macro logic: has_serde = serialize || deserialize
        self.has_request || self.has_response
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
