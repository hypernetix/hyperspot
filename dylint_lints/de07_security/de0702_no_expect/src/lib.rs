#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_hir;

use rustc_hir::{Expr, ExprKind};
use rustc_lint::{LateContext, LateLintPass, LintContext};

dylint_linting::declare_late_lint! {
    /// ### What it does
    ///
    /// Detects usage of the `.expect()` method on `Option` and `Result` types.
    ///
    /// ### Why is this bad?
    ///
    /// Using `.expect()` can cause a panic at runtime if the value is `None` or `Err`.
    /// In production code, panics should be avoided in favor of proper error handling.
    /// While `.expect()` provides a message, it still crashes the program.
    ///
    /// ### Example
    ///
    /// ```rust,ignore
    /// // Bad - can panic
    /// let value = some_option.expect("value should exist");
    /// let data = result.expect("operation should succeed");
    /// ```
    ///
    /// Use instead:
    ///
    /// ```rust,ignore
    /// // Good - explicit error handling
    /// let value = some_option.ok_or(MyError::NotFound)?;
    /// let data = result.map_err(|e| MyError::from(e))?;
    /// ```
    pub DE0702_NO_EXPECT,
    Warn,
    "avoid using .expect() - use proper error handling instead (DE0702)"
}

impl<'tcx> LateLintPass<'tcx> for De0702NoExpect {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'tcx>) {
        if let ExprKind::MethodCall(method_segment, _receiver, _args, span) = &expr.kind {
            let method_name = method_segment.ident.name.as_str();

            if method_name == "expect" {
                cx.span_lint(DE0702_NO_EXPECT, *span, |diag| {
                    diag.primary_message("avoid using .expect() (DE0702)");
                    diag.help("use .ok_or()?, .map_err()?, match, or if let instead");
                    diag.note("expect() can panic at runtime; use explicit error handling for robustness");
                });
            }
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
        lint_utils::test_comment_annotations_match_stderr(&ui_dir, "DE0702", "no expect");
    }
}
