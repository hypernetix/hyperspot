#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_hir;
extern crate rustc_span;

use clippy_utils::macros::root_macro_call_first_node;
use lint_utils::is_in_domain_path;
use rustc_hir::Expr;
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_span::sym;

dylint_linting::declare_late_lint! {
    /// ### What it does
    ///
    /// Forbids usage of `panic!()`, `todo!()`, `unreachable!()`, and `unimplemented!()`
    /// macros in domain layer code.
    ///
    /// ### Why is this bad?
    ///
    /// Domain logic should be pure and handle errors gracefully through Result types:
    /// - `panic!()` - abruptly terminates the program
    /// - `todo!()` - indicates incomplete code that shouldn't reach production
    /// - `unreachable!()` - if reached, indicates a logic error
    /// - `unimplemented!()` - indicates missing functionality
    ///
    /// Domain code should return Result<T, E> for proper error propagation.
    ///
    /// ### Example
    ///
    /// ```rust,ignore
    /// // Bad - panics in domain layer
    /// mod domain {
    ///     fn validate(data: &str) -> Value {
    ///         if data.is_empty() {
    ///             panic!("data cannot be empty");
    ///         }
    ///         todo!("implement validation")
    ///     }
    /// }
    /// ```
    ///
    /// Use instead:
    ///
    /// ```rust,ignore
    /// // Good - proper error handling in domain
    /// mod domain {
    ///     fn validate(data: &str) -> Result<Value, DomainError> {
    ///         if data.is_empty() {
    ///             return Err(DomainError::EmptyInput);
    ///         }
    ///         Err(DomainError::NotImplemented("validation"))
    ///     }
    /// }
    /// ```
    pub DE0305_NO_PANICS_IN_DOMAIN,
    Warn,
    "forbids panic!(), todo!(), unreachable!(), unimplemented!() in domain layer (DE0305)"
}

impl<'tcx> LateLintPass<'tcx> for De0305NoPanicsInDomain {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'tcx>) {
        // Check if this is a panic-like macro
        let Some(macro_call) = root_macro_call_first_node(cx, expr) else {
            return;
        };

        let macro_name = if cx.tcx.is_diagnostic_item(sym::core_panic_macro, macro_call.def_id) {
            "panic"
        } else if cx.tcx.is_diagnostic_item(sym::std_panic_macro, macro_call.def_id) {
            "panic"
        } else if cx.tcx.is_diagnostic_item(sym::todo_macro, macro_call.def_id) {
            "todo"
        } else if cx.tcx.is_diagnostic_item(sym::unreachable_macro, macro_call.def_id) {
            "unreachable"
        } else if cx.tcx.is_diagnostic_item(sym::unimplemented_macro, macro_call.def_id) {
            "unimplemented"
        } else {
            return;
        };

        // Only check files in domain layer
        if !is_in_domain_path(cx.sess().source_map(), macro_call.span) {
            return;
        }

        cx.span_lint(DE0305_NO_PANICS_IN_DOMAIN, macro_call.span, |diag| {
            diag.primary_message(format!(
                "`{}!()` macro should not be used in domain layer (DE0305)",
                macro_name
            ));
            diag.help("use Result<T, E> for error handling instead of panicking");
            diag.note("domain logic should be pure and handle errors gracefully");
        });
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
            "DE0305",
            "panic in domain"
        );
    }
}
