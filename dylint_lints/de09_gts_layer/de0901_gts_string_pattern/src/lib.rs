#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;
extern crate rustc_span;

use rustc_ast::{AttrKind, Attribute, Expr, ExprKind};
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};

dylint_linting::declare_pre_expansion_lint! {
    /// ### What it does
    ///
    /// Validates GTS schema identifiers used by `gts-macros`.
    ///
    /// Specifically checks `schema_id = "..."` inside `#[struct_to_gts_schema(...)]`.
    pub DE0901_GTS_STRING_PATTERN,
    Deny,
    "invalid GTS schema_id string (DE0901)"
}

impl EarlyLintPass for De0901GtsStringPattern {
    fn check_attribute(&mut self, cx: &EarlyContext<'_>, attr: &Attribute) {
        self.check_struct_to_gts_schema_attr(cx, attr);
    }

    fn check_expr(&mut self, cx: &EarlyContext<'_>, expr: &Expr) {
        self.check_string_literal_expr(cx, expr);
    }
}

impl De0901GtsStringPattern {
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

            if mi.path.segments.len() != 1 || mi.path.segments[0].ident.name.as_str() != "schema_id" {
                continue;
            }

            let Some(val) = mi.value_str() else {
                continue;
            };

            self.check_schema_id_string(cx, mi.span, val.as_str());
        }
    }

    fn check_string_literal_expr(&self, cx: &EarlyContext<'_>, expr: &Expr) {
        let ExprKind::Lit(lit) = &expr.kind else {
            return;
        };

        match lit.kind {
            rustc_ast::token::LitKind::Str | rustc_ast::token::LitKind::StrRaw(_) => {
                // `lit.symbol` includes the quotes. Remove them.
                let raw = lit.symbol.as_str();
                let Some(s) = raw.strip_prefix('"').and_then(|v| v.strip_suffix('"')) else {
                    return;
                };

                if s.trim_start().starts_with("gts.") {
                    self.check_schema_id_string(cx, expr.span, s);
                }
            }
            _ => {}
        }
    }

    fn check_schema_id_string(&self, cx: &EarlyContext<'_>, span: rustc_span::Span, s: &str) {
        // Real gts-macros schema_id format (see .gts-rust/gts-macros/README.md):
        // - First segment begins with `gts.` and ends with `~`
        // - Additional segments (inheritance chain) are separated by `~` and each ends with `~`
        //   but do NOT repeat the `gts.` prefix.
        //
        // Example:
        // gts.x.core.events.type.v1~x.core.audit.event.v1~x.marketplace.orders.purchase.v1~
        let pattern = r"^\s*gts\.[a-z_][a-z0-9_]*\.[a-z_][a-z0-9_]*\.[a-z_][a-z0-9_]*\.[a-z_][a-z0-9_]*\.v(0|[1-9]\d*)(?:\.(0|[1-9]\d*))?~(?:[a-z_][a-z0-9_]*\.[a-z_][a-z0-9_]*\.[a-z_][a-z0-9_]*\.[a-z_][a-z0-9_]*\.v(0|[1-9]\d*)(?:\.(0|[1-9]\d*))?~)*\s*$";

        let re = regex::Regex::new(pattern).unwrap();
        if !re.is_match(s) {
            cx.span_lint(DE0901_GTS_STRING_PATTERN, span, |diag| {
                diag.primary_message(format!("invalid GTS schema_id string: '{}' (DE0901)", s));
                diag.note("Expected gts-macros schema_id format: gts.<v>.<p>.<ns>.<name>.v<MAJOR>[.<MINOR>]~ with optional ~segment~ chain");
                diag.help("Examples:");
                diag.help("  gts.x.core.events.type.v1~");
                diag.help("  gts.x.core.events.type.v1~x.core.audit.event.v1~");
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
            "invalid GTS schema_id string"
        );
    }
}
