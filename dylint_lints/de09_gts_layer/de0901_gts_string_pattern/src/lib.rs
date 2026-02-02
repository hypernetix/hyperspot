#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;
extern crate rustc_span;

use gts::{GtsIdSegment, GtsOps};
use rustc_ast::token::LitKind;
use rustc_ast::{AttrKind, Attribute, Expr, ExprKind};
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};
use rustc_span::Span;
use std::cell::RefCell;
use std::collections::HashSet;

// Thread-local storage for spans to skip (inside starts_with calls)
thread_local! {
    static SKIP_SPANS: RefCell<HashSet<Span>> = RefCell::new(HashSet::new());
}

dylint_linting::declare_pre_expansion_lint! {
    /// ### What it does
    ///
    /// Validates GTS schema identifiers used by `gts-macros`.
    ///
    /// Checks:
    /// 1. `schema_id = "..."` in `#[struct_to_gts_schema(...)]` - must be valid GTS type schema
    /// 2. `gts_make_instance_id("...")` - must be valid GTS instance segment id
    /// 3. GTS-looking string literals - must be valid GTS entity id
    ///
    /// Uses `GtsOps::parse_id()` from the GTS library for validation.
    pub DE0901_GTS_STRING_PATTERN,
    Deny,
    "invalid GTS string pattern (DE0901)"
}

impl EarlyLintPass for De0901GtsStringPattern {
    fn check_attribute(&mut self, cx: &EarlyContext<'_>, attr: &Attribute) {
        self.check_struct_to_gts_schema_attr(cx, attr);
    }

    fn check_expr(&mut self, cx: &EarlyContext<'_>, expr: &Expr) {
        // First, collect spans from starts_with calls to skip
        // This runs before checking, so nested expressions will be marked
        if let ExprKind::MethodCall(method_call) = &expr.kind {
            let method_name = method_call.seg.ident.name.as_str();
            if method_name == "starts_with" {
                // Add the receiver and all arguments to skip list
                SKIP_SPANS.with(|spans| {
                    let mut spans = spans.borrow_mut();
                    spans.insert(method_call.receiver.span);
                    for arg in &method_call.args {
                        spans.insert(arg.span);
                    }
                });
                // Don't check anything in starts_with calls
                return;
            }
            if method_name == "resource_pattern" || method_name == "with_pattern" {
                // Add arguments to skip list - wildcards are allowed in these methods
                SKIP_SPANS.with(|spans| {
                    let mut spans = spans.borrow_mut();
                    for arg in &method_call.args {
                        spans.insert(arg.span);
                    }
                });
            }
        }

        // Check if this expression should be skipped (it's inside a starts_with call)
        let should_skip = SKIP_SPANS.with(|spans| spans.borrow().contains(&expr.span));
        if should_skip {
            return;
        }

        self.check_gts_make_instance_id_call(cx, expr);

        // Check if this is a method call - handle resource_pattern and with_pattern specially
        if let ExprKind::MethodCall(method_call) = &expr.kind {
            let method_name = method_call.seg.ident.name.as_str();
            // Check if this is a method call to resource_pattern or with_pattern - allow wildcards in its arguments
            if method_name == "resource_pattern" || method_name == "with_pattern" {
                // Check string literals in these calls with wildcards allowed
                for arg in &method_call.args {
                    self.check_gts_string_literal_with_wildcards(cx, arg);
                }
                return;
            }

            // Check arguments of other method calls normally
            for arg in &method_call.args {
                self.check_gts_string_literal(cx, arg);
            }
            return;
        }

        // For non-method-call expressions, check normally
        self.check_gts_string_literal(cx, expr);
    }
}

impl De0901GtsStringPattern {
    fn check_gts_make_instance_id_call(&self, cx: &EarlyContext<'_>, expr: &Expr) {
        let ExprKind::Call(func, args) = &expr.kind else {
            return;
        };

        if args.len() != 1 {
            return;
        }

        let Some(arg0) = args.get(0) else {
            return;
        };

        let Some(arg_str) = Self::string_lit_value(arg0) else {
            return;
        };

        // Detect `...::gts_make_instance_id("...")`
        let ExprKind::Path(_, path) = &func.kind else {
            return;
        };

        let Some(last) = path.segments.last() else {
            return;
        };

        if last.ident.name.as_str() != "gts_make_instance_id" {
            return;
        }

        self.validate_instance_id_segment(cx, expr.span, arg_str);
    }

    fn check_gts_string_literal(&self, cx: &EarlyContext<'_>, expr: &Expr) {
        self.check_gts_string_literal_with_wildcard_flag(cx, expr, false);
    }

    fn check_gts_string_literal_with_wildcards(&self, cx: &EarlyContext<'_>, expr: &Expr) {
        self.check_gts_string_literal_with_wildcard_flag(cx, expr, true);
    }

    fn check_gts_string_literal_with_wildcard_flag(
        &self,
        cx: &EarlyContext<'_>,
        expr: &Expr,
        allow_wildcards: bool,
    ) {
        if let Some(s) = Self::string_lit_value(expr) {
            let s = s.trim();

            // Option 1: String starts with "gts." - validate directly
            if s.starts_with("gts.") {
                if allow_wildcards {
                    self.validate_any_gts_id_allow_wildcards(cx, expr.span, s);
                } else {
                    self.validate_any_gts_id(cx, expr.span, s);
                }
                return;
            }

            // Option 2: String contains ":" - this is a permission string format
            // Permission strings ALWAYS allow wildcards in their GTS parts
            if s.contains(':') {
                for part in s.split(':') {
                    if part.trim().starts_with("gts.") {
                        self.validate_any_gts_id_allow_wildcards(cx, expr.span, part.trim());
                        break; // Only validate the first GTS part found
                    }
                }
            }
        }
    }

    fn string_lit_value(expr: &Expr) -> Option<&str> {
        match &expr.kind {
            ExprKind::Lit(lit) => match lit.kind {
                LitKind::Str | LitKind::StrRaw(_) => Some(lit.symbol.as_str()),
                _ => None,
            },
            _ => None,
        }
    }

    fn check_struct_to_gts_schema_attr(&self, cx: &EarlyContext<'_>, attr: &Attribute) {
        let AttrKind::Normal(normal_attr) = &attr.kind else {
            return;
        };

        // We only care about #[struct_to_gts_schema(...)]
        if normal_attr.item.path.segments.len() != 1
            || normal_attr.item.path.segments[0].ident.name.as_str() != "struct_to_gts_schema"
        {
            return;
        }

        let Some(items) = normal_attr.item.meta_item_list() else {
            return;
        };

        for nested in items {
            let Some(mi) = nested.meta_item() else {
                continue;
            };

            if mi.path.segments.len() != 1 || mi.path.segments[0].ident.name.as_str() != "schema_id"
            {
                continue;
            }

            let Some(val) = mi.value_str() else {
                continue;
            };

            self.validate_schema_id(cx, mi.span, val.as_str());
        }
    }

    /// Validate a GTS schema_id using GtsOps::parse_id()
    /// schema_id must be a valid GTS type schema (ending with ~)
    fn validate_schema_id(&self, cx: &EarlyContext<'_>, span: rustc_span::Span, s: &str) {
        let s = s.trim();

        // Wildcards are NOT allowed in schema_id
        if s.contains('*') {
            cx.span_lint(DE0901_GTS_STRING_PATTERN, span, |diag| {
                diag.primary_message(format!("wildcards are not allowed in schema_id: '{}' (DE0901)", s));
                diag.note("Wildcards (*) are only allowed in permission strings, not in schema_id attributes");
                diag.help("Use concrete type names in schema_id");
            });
            return;
        }

        // Use GtsOps::parse_id() for validation - it gives us parsed segments
        let ops = GtsOps::new(None, None, 0);
        let result = ops.parse_id(s);

        if !result.ok {
            cx.span_lint(DE0901_GTS_STRING_PATTERN, span, |diag| {
                diag.primary_message(format!("invalid GTS schema_id: '{}' (DE0901)", s));
                diag.note(result.error);
                diag.help("Example: gts.x.core.events.type.v1~");
            });
            return;
        }

        // Ensure it's actually a schema (type), not an instance
        if result.is_schema != Some(true) {
            cx.span_lint(DE0901_GTS_STRING_PATTERN, span, |diag| {
                diag.primary_message(format!(
                    "schema_id must be a type schema, not an instance: '{}' (DE0901)",
                    s
                ));
                diag.note("schema_id must end with '~' to indicate it's a type schema");
                diag.help("Example: gts.x.core.events.type.v1~");
            });
        }
    }

    fn validate_instance_id_segment(&self, cx: &EarlyContext<'_>, span: rustc_span::Span, s: &str) {
        let s = s.trim();

        // `gts_make_instance_id` accepts a single *segment id* (no `gts.` prefix),
        // so we must not validate it as a full GTS ID.
        // If the input contains delimiters for chained ids / permission strings,
        // it is not a single segment.
        if s.contains('~') || s.contains(':') {
            cx.span_lint(DE0901_GTS_STRING_PATTERN, span, |diag| {
                diag.primary_message(format!(
                    "gts_make_instance_id expects a single GTS segment, got: '{}' (DE0901)",
                    s
                ));
                diag.help("Example: vendor.package.sku.abc.v1");
            });
            return;
        }

        if s.contains('*') {
            cx.span_lint(DE0901_GTS_STRING_PATTERN, span, |diag| {
                diag.primary_message(format!(
                    "wildcards are not allowed in instance id segments: '{}' (DE0901)",
                    s
                ));
                diag.help("Example: vendor.package.sku.abc.v1");
            });
            return;
        }

        if let Err(e) = GtsIdSegment::new(0, 0, s) {
            cx.span_lint(DE0901_GTS_STRING_PATTERN, span, |diag| {
                diag.primary_message(format!("invalid GTS segment: '{}' (DE0901)", s));
                diag.note(e.to_string());
                diag.help("Example: vendor.package.sku.abc.v1");
            });
        }
    }

    fn validate_any_gts_id(&self, cx: &EarlyContext<'_>, span: rustc_span::Span, s: &str) {
        let s = s.trim();

        // Wildcards are NOT allowed in regular GTS strings (only in permission strings)
        if s.contains('*') {
            cx.span_lint(DE0901_GTS_STRING_PATTERN, span, |diag| {
                diag.primary_message(format!("invalid GTS string (wildcards not allowed): '{}' (DE0901)", s));
                diag.note("Wildcards (*) are only allowed in permission strings, not in regular GTS identifiers");
                diag.help("Use concrete type names");
            });
            return;
        }

        let ops = GtsOps::new(None, None, 0);
        let result = ops.parse_id(s);

        if !result.ok {
            cx.span_lint(DE0901_GTS_STRING_PATTERN, span, |diag| {
                diag.primary_message(format!("invalid GTS string: '{}' (DE0901)", s));
                diag.note(result.error);
            });
        }
    }

    fn validate_any_gts_id_allow_wildcards(
        &self,
        cx: &EarlyContext<'_>,
        span: rustc_span::Span,
        s: &str,
    ) {
        let s = s.trim();

        // For resource_pattern calls, we allow wildcards but still validate the GTS structure
        let ops = GtsOps::new(None, None, 0);
        let result = ops.parse_id(s);

        if !result.ok {
            cx.span_lint(DE0901_GTS_STRING_PATTERN, span, |diag| {
                diag.primary_message(format!("invalid GTS string: '{}' (DE0901)", s));
                diag.note(result.error);
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
            "DE0901",
            "invalid GTS", // Matches both "invalid GTS string" and "invalid GTS schema_id string"
        );
    }
}
