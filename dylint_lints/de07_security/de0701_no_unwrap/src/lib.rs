#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_hir;

use rustc_hir::{Expr, ExprKind};
use rustc_lint::{LateContext, LateLintPass, LintContext};

dylint_linting::declare_late_lint! {
    /// ### What it does
    ///
    /// Detects usage of the `.unwrap()` method on `Option` and `Result` types.
    ///
    /// ### Why is this bad?
    ///
    /// Using `.unwrap()` can cause a panic at runtime if the value is `None` or `Err`.
    /// In production code, panics should be avoided in favor of proper error handling.
    ///
    /// ### Example
    ///
    /// ```rust,ignore
    /// // Bad - can panic
    /// let value = some_option.unwrap();
    /// let data = result.unwrap();
    /// ```
    ///
    /// Use instead:
    ///
    /// ```rust,ignore
    /// // Good - explicit error handling
    /// let value = some_option.ok_or(MyError::NotFound)?;
    /// let data = result.map_err(|e| MyError::from(e))?;
    /// 
    /// // Or with match
    /// let value = match some_option {
    ///     Some(v) => v,
    ///     None => return Err(MyError::NotFound),
    /// };
    /// ```
    pub DE0701_NO_UNWRAP,
    Deny,
    "avoid using .unwrap() - use proper error handling instead (DE0701)"
}

impl<'tcx> LateLintPass<'tcx> for De0701NoUnwrap {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'tcx>) {
        // Look for method calls named "unwrap"
        if let ExprKind::MethodCall(method_segment, _receiver, _args, span) = &expr.kind {
            let method_name = method_segment.ident.name.as_str();
            
            if method_name == "unwrap" {
                cx.span_lint(DE0701_NO_UNWRAP, *span, |diag| {
                    diag.primary_message("avoid using .unwrap() (DE0701)");
                    diag.help("use .ok_or()?, .map_err()?, match, or if let instead");
                    diag.note("unwrap() can panic at runtime; use explicit error handling for robustness");
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
        lint_utils::test_comment_annotations_match_stderr(&ui_dir, "DE0701", "no unwrap");
    }
}
