//! Utility functions for dylint linters

use rustc_hir::def_id::LocalDefId;
use rustc_hir::Item;
use rustc_lint::{EarlyContext, LateContext, LintContext};
use rustc_span::Span;

/// Check if the given definition is in a `contract` module
/// Matches both absolute and relative paths
pub fn is_in_contract_module(cx: &LateContext<'_>, def_id: LocalDefId) -> bool {
    is_in_path_containing(cx, def_id, "contract/")
}

/// Check if the given definition is in a `domain` module
pub fn is_in_domain_module(cx: &LateContext<'_>, def_id: LocalDefId) -> bool {
    is_in_path_containing(cx, def_id, "domain/")
}

/// Check if the given definition is in an `infra` module
pub fn is_in_infra_module(cx: &LateContext<'_>, def_id: LocalDefId) -> bool {
    is_in_path_containing(cx, def_id, "infra/")
}

/// Check if the given definition is in an `api/rest` folder
pub fn is_in_api_rest_folder(cx: &LateContext<'_>, def_id: LocalDefId) -> bool {
    is_in_path_containing(cx, def_id, "api/rest/")
}

/// Check if the given span is in an `api/rest` folder (EarlyContext version)
pub fn is_in_api_rest_folder_early(cx: &EarlyContext<'_>, span: Span) -> bool {
    check_span_path_containing(cx, span, "api/rest/")
}

/// Check if the given definition is in a module crate (modules/ folder)
/// This is used to exempt library crates from certain checks
pub fn is_in_module_crate(cx: &LateContext<'_>, def_id: LocalDefId) -> bool {
    is_in_path_containing(cx, def_id, "modules/")
}

/// Check if the file path contains the given substring
/// Handles both Unix and Windows path separators
fn is_in_path_containing(cx: &LateContext<'_>, def_id: LocalDefId, pattern: &str) -> bool {
    let source_map = cx.tcx.sess.source_map();
    let span = cx.tcx.def_span(def_id);
    check_span_path(source_map, span, pattern)
}

/// Check if the file path contains the given substring (EarlyContext version)
fn check_span_path_containing(cx: &EarlyContext<'_>, span: Span, pattern: &str) -> bool {
    let source_map = cx.sess().source_map();
    check_span_path(source_map, span, pattern)
}

/// Helper to check path string
fn check_span_path(
    source_map: &rustc_span::source_map::SourceMap,
    span: Span,
    pattern: &str,
) -> bool {
    let filename = source_map.span_to_filename(span);
    let filename_display = filename.prefer_local();
    let path_str = format!("{}", filename_display);

    // Check for Unix and Windows path separators
    let pattern_windows = pattern.replace('/', "\\");
    path_str.contains(pattern) || path_str.contains(&pattern_windows)
}

/// Get item name from the compiler context
pub fn get_item_name(cx: &LateContext<'_>, item: &Item<'_>) -> String {
    cx.tcx.item_name(item.owner_id.to_def_id()).to_string()
}

/// Get item name from AST item (for EarlyLint passes)
/// Returns the identifier name as a string
/// In newer rustc versions, the ident is stored within ItemKind variants
pub fn get_ast_item_name(item: &rustc_ast::Item) -> String {
    use rustc_ast::ItemKind;
    match &item.kind {
        // ItemKind::Struct(ident, generics, variant_data)
        // ident.name is a Symbol, which has as_str()
        ItemKind::Struct(ident, _, _) => ident.name.as_str().to_string(),
        // ItemKind::Enum(ident, generics, enum_def)
        ItemKind::Enum(ident, _, _) => ident.name.as_str().to_string(),
        // ItemKind::Union(ident, generics, variant_data)
        ItemKind::Union(ident, _, _) => ident.name.as_str().to_string(),
        // For other item kinds, return empty string
        _ => String::new(),
    }
}

/// Convert a use path to a string for pattern matching
pub fn path_to_string(path: &rustc_hir::UsePath<'_>) -> String {
    path.segments
        .iter()
        .map(|seg| seg.ident.name.as_str())
        .collect::<Vec<_>>()
        .join("::")
}
