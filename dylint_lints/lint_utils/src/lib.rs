#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_span;

use rustc_hir::def_id::LocalDefId;
use rustc_hir::Item;
use rustc_lint::{EarlyContext, LateContext, LintContext};
use rustc_span::Span;

/// Extract simulated directory path from a comment at the start of a file.
/// Looks for a comment like: `// simulated_dir=/hyperspot/modules/some_module/contract/`
/// Returns None if no such comment is found.
/// 
/// Only checks files in temporary directories to avoid unnecessary file I/O in production.
fn extract_simulated_dir(source_map: &rustc_span::source_map::SourceMap, span: Span) -> Option<String> {
    let file_name = source_map.span_to_filename(span);
    
    use rustc_span::FileName;
    let local_path = match file_name {
        FileName::Real(ref real_name) => real_name.local_path()?,
        _ => return None,
    };
    
    // Only check for simulated_dir in temporary paths (tests run in temp directories)
    let path_str = local_path.to_string_lossy();
    let is_temp = path_str.contains("/tmp/") 
        || path_str.contains("/var/folders/")  // macOS temp
        || path_str.contains("\\Temp\\")        // Windows temp
        || path_str.contains(".tmp");           // dylint test temp dirs
    
    if !is_temp {
        return None;
    }
    
    // Read the first few lines of the file to check for simulated_dir comment
    let contents = std::fs::read_to_string(local_path).ok()?;
    
    for line in contents.lines().take(10) {
        let trimmed = line.trim();
        if trimmed.starts_with("// simulated_dir=") {
            return Some(trimmed.trim_start_matches("// simulated_dir=").to_string());
        }
        // Stop at first non-comment, non-empty line
        if !trimmed.is_empty() && !trimmed.starts_with("//") && !trimmed.starts_with("#!") {
            break;
        }
    }
    
    None
}

pub fn is_in_contract_module(cx: &LateContext<'_>, def_id: LocalDefId) -> bool {
    is_in_module_named(cx, def_id, "contract") || is_in_path_containing(cx, def_id, "contract/")
}

pub fn is_in_domain_module(cx: &LateContext<'_>, def_id: LocalDefId) -> bool {
    is_in_module_named(cx, def_id, "domain") || is_in_path_containing(cx, def_id, "domain/")
}

pub fn is_in_infra_module(cx: &LateContext<'_>, def_id: LocalDefId) -> bool {
    is_in_module_named(cx, def_id, "infra") || is_in_path_containing(cx, def_id, "infra/")
}

/// Check if a def_id is within a module with the given name in the HIR hierarchy
/// This handles inline modules like `mod contract { ... }`
fn is_in_module_named(cx: &LateContext<'_>, def_id: LocalDefId, module_name: &str) -> bool {
    let def_path = cx.tcx.def_path(def_id.to_def_id());
    
    for component in def_path.data.iter() {
        if let rustc_hir::definitions::DefPathData::TypeNs(symbol) = component.data
            && symbol.as_str() == module_name {
            return true;
        }
    }
    
    false
}

pub fn is_in_api_rest_folder(cx: &LateContext<'_>, def_id: LocalDefId) -> bool {
    is_in_path_containing(cx, def_id, "api/rest/")
}

pub fn is_in_api_rest_folder_early(cx: &EarlyContext<'_>, span: Span) -> bool {
    check_span_path_containing(cx, span, "api/rest/")
}

pub fn is_in_contract_module_ast(cx: &EarlyContext<'_>, item: &rustc_ast::Item) -> bool {
    let source_map = cx.sess().source_map();
    
    // Check for simulated directory in test files first
    if let Some(simulated) = extract_simulated_dir(source_map, item.span) {
        return simulated.contains("/contract/") || simulated.contains("\\contract\\");
    }
    
    // Fall back to actual file path
    let filename = source_map.span_to_filename(item.span);
    let path_str = format!("{:?}", filename);
    
    path_str.contains("/contract/") || path_str.contains("\\contract\\")
}

/// Check if an AST item is in an api/rest folder
/// Checks the source file path for */api/rest/* pattern
pub fn is_in_api_rest_folder_ast(cx: &EarlyContext<'_>, item: &rustc_ast::Item) -> bool {
    use rustc_span::FileName;
    
    let source_map = cx.sess().source_map();
    
    // Check for simulated directory in test files first
    if let Some(simulated) = extract_simulated_dir(source_map, item.span) {
        return simulated.contains("/api/rest/") || simulated.contains("\\api\\rest\\");
    }
    
    // Fall back to actual file path
    let file_name = source_map.span_to_filename(item.span);
    
    let path_str = match file_name {
        FileName::Real(ref real_name) => {
            if let Some(local) = real_name.local_path() {
                local.to_string_lossy().to_string()
            } else {
                return false;
            }
        }
        _ => return false,
    };
    
    path_str.contains("/api/rest/") || path_str.contains("\\api\\rest\\")
}

pub fn is_in_module_crate(cx: &LateContext<'_>, def_id: LocalDefId) -> bool {
    is_in_path_containing(cx, def_id, "modules/")
}

fn is_in_path_containing(cx: &LateContext<'_>, def_id: LocalDefId, pattern: &str) -> bool {
    let source_map = cx.tcx.sess.source_map();
    let span = cx.tcx.def_span(def_id);
    check_span_path(source_map, span, pattern)
}

fn check_span_path_containing(cx: &EarlyContext<'_>, span: Span, pattern: &str) -> bool {
    let source_map = cx.sess().source_map();
    check_span_path(source_map, span, pattern)
}

fn check_span_path(
    source_map: &rustc_span::source_map::SourceMap,
    span: Span,
    pattern: &str,
) -> bool {
    let filename = source_map.span_to_filename(span);
    let path_str = format!("{filename:?}");

    let pattern_windows = pattern.replace('/', "\\");
    path_str.contains(pattern) || path_str.contains(&pattern_windows)
}

pub fn get_item_name(cx: &LateContext<'_>, item: &Item<'_>) -> String {
    cx.tcx.item_name(item.owner_id.to_def_id()).to_string()
}

pub fn get_ast_item_name(item: &rustc_ast::Item) -> String {
    use rustc_ast::ItemKind;

    match &item.kind {
        ItemKind::Struct(ident, _, _) => ident.name.as_str().to_string(),
        ItemKind::Enum(ident, _, _) => ident.name.as_str().to_string(),
        ItemKind::Union(ident, _, _) => ident.name.as_str().to_string(),
        _ => String::new(),
    }
}

pub fn path_to_string(path: &rustc_hir::UsePath<'_>) -> String {
    path.segments
        .iter()
        .map(|seg| seg.ident.name.as_str())
        .collect::<Vec<_>>()
        .join("::")
}

/// Generic helper to traverse inline "contract" modules and apply a check function to items within.
/// Returns true if the item was a contract module (and was handled), false otherwise.
///
/// This handles the pattern:
/// ```rust
/// mod contract {
///     // items here will be passed to check_fn
/// }
/// ```
pub fn for_each_item_in_contract_module<F>(
    cx: &EarlyContext<'_>,
    item: &rustc_ast::Item,
    mut check_fn: F,
) -> bool
where
    F: FnMut(&EarlyContext<'_>, &rustc_ast::Item),
{
    use rustc_ast::ItemKind;

    if let ItemKind::Mod(_, ident, mod_kind) = &item.kind
        && ident.name.as_str() == "contract" {
        if let rustc_ast::ModKind::Loaded(items, ..) = mod_kind {
            for inner_item in items {
                check_fn(cx, inner_item);
            }
        }
        return true;
    }
    false
}

/// Check if path segments represent a serde trait (Serialize or Deserialize)
/// 
/// Handles various forms:
/// - Bare: `Serialize`, `Deserialize`
/// - Qualified: `serde::Serialize`, `serde::Deserialize`
/// - Fully qualified: `::serde::Serialize`
/// 
/// # Examples
/// ```no_run
/// use lint_utils::is_serde_trait;
/// assert!(is_serde_trait(&["Serialize"], "Serialize"));
/// assert!(is_serde_trait(&["serde", "Serialize"], "Serialize"));
/// assert!(is_serde_trait(&["serde", "Deserialize"], "Deserialize"));
/// assert!(!is_serde_trait(&["other", "Serialize"], "Serialize"));
/// ```
pub fn is_serde_trait(segments: &[&str], trait_name: &str) -> bool {
    if segments.is_empty() {
        return false;
    }
    
    // Check if last segment matches the trait name
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
