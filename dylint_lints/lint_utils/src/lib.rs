#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_span;

use rustc_hir::def_id::LocalDefId;
use rustc_hir::Item;
use rustc_lint::{EarlyContext, LateContext, LintContext};
use rustc_span::Span;

pub fn is_in_contract_module(cx: &LateContext<'_>, def_id: LocalDefId) -> bool {
    is_in_path_containing(cx, def_id, "contract/")
}

pub fn is_in_domain_module(cx: &LateContext<'_>, def_id: LocalDefId) -> bool {
    is_in_path_containing(cx, def_id, "domain/")
}

pub fn is_in_infra_module(cx: &LateContext<'_>, def_id: LocalDefId) -> bool {
    is_in_path_containing(cx, def_id, "infra/")
}

pub fn is_in_api_rest_folder(cx: &LateContext<'_>, def_id: LocalDefId) -> bool {
    is_in_path_containing(cx, def_id, "api/rest/")
}

pub fn is_in_api_rest_folder_early(cx: &EarlyContext<'_>, span: Span) -> bool {
    check_span_path_containing(cx, span, "api/rest/")
}

pub fn is_in_contract_module_ast(cx: &EarlyContext<'_>, item: &rustc_ast::Item) -> bool {
    let source_map = cx.sess().source_map();
    let filename = source_map.span_to_filename(item.span);
    let path_str = format!("{:?}", filename);
    
    // Check if the full path contains /contract/ or \contract\
    // This covers both:
    // 1. Files in a contract/ directory: /path/to/contract/file.rs
    // 2. Files in ui/contract/ for tests: /path/to/ui/contract/file.rs
    // Note: Due to how compiletest flattens paths, UI tests wrap code in `mod contract {}`
    // which is semantically equivalent for testing purposes
    path_str.contains("/contract/") || path_str.contains("\\contract\\")
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
