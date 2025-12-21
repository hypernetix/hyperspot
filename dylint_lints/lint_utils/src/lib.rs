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
