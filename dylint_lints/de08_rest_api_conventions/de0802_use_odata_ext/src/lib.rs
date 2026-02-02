#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;
extern crate rustc_hir;

use rustc_hir::{Expr, ExprKind};
use rustc_lint::{LateContext, LateLintPass, LintContext};

dylint_linting::declare_late_lint! {
    /// ### What it does
    ///
    /// Checks that OData query parameters (`$filter`, `$orderby`, `$select`, `$top`, `$skip`)
    /// are registered using `OperationBuilderODataExt` methods instead of manual `.query_param()` calls.
    ///
    /// ### Why is this bad?
    ///
    /// Using `.query_param("$filter", ...)` bypasses the type-safe OData system:
    /// - No compile-time validation of filterable/orderable fields
    /// - No automatic OpenAPI schema generation for allowed fields
    /// - Inconsistent API documentation
    /// - Harder to maintain as DTO fields change
    ///
    /// ### Example
    ///
    /// ```rust,ignore
    /// // Bad - manual OData parameter registration
    /// OperationBuilder::get("/users-info/v1/users")
    ///     .query_param("$filter", false, "OData filter")
    ///     .query_param("$orderby", false, "OData ordering")
    ///     .query_param("$select", false, "OData field selection")
    /// ```
    ///
    /// Use instead:
    ///
    /// ```rust,ignore
    /// // Good - type-safe OData registration
    /// OperationBuilder::get("/users-info/v1/users")
    ///     .with_odata_filter::<dto::UserDtoFilterField>()
    ///     .with_odata_orderby::<dto::UserDtoFilterField>()
    ///     .with_odata_select()
    /// ```
    pub DE0802_USE_ODATA_EXT,
    Deny,
    "use OperationBuilderODataExt methods instead of .query_param() for OData parameters (DE0802)"
}

/// OData query parameter names that should use the type-safe extension methods
const ODATA_PARAMS: &[&str] = &["$filter", "$orderby", "$select", "$top", "$skip", "$count"];

/// Mapping from OData parameter to the recommended method
fn get_recommended_method(param: &str) -> &'static str {
    match param {
        "$filter" => ".with_odata_filter::<FilterFieldEnum>()",
        "$orderby" => ".with_odata_orderby::<FilterFieldEnum>()",
        "$select" => ".with_odata_select()",
        "$top" | "$skip" | "$count" => ".query_param_typed() with proper OData extractor",
        _ => "the appropriate OperationBuilderODataExt method",
    }
}

impl<'tcx> LateLintPass<'tcx> for De0802UseOdataExt {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'tcx>) {
        // Look for method calls like .query_param(...) or .query_param_typed(...)
        if let ExprKind::MethodCall(method_segment, receiver, args, _span) = &expr.kind {
            let method_name = method_segment.ident.name.as_str();

            // Check if this is a query_param or query_param_typed call
            if method_name != "query_param" && method_name != "query_param_typed" {
                return;
            }

            // Check if the receiver chain contains OperationBuilder
            if !is_operation_builder_chain(receiver) {
                return;
            }

            // Check the first argument (parameter name)
            if let Some(first_arg) = args.first() {
                check_odata_param(cx, first_arg, method_name);
            }
        }
    }
}

/// Check if an expression is part of an OperationBuilder method chain
fn is_operation_builder_chain(expr: &Expr<'_>) -> bool {
    match &expr.kind {
        // Direct call like OperationBuilder::get(...)
        ExprKind::Call(func, _) => {
            if let ExprKind::Path(qpath) = &func.kind {
                return path_contains_operation_builder(qpath);
            }
            false
        }
        // Method chain like builder.something().query_param(...)
        ExprKind::MethodCall(_, receiver, _, _) => is_operation_builder_chain(receiver),
        // Path expression
        ExprKind::Path(qpath) => path_contains_operation_builder(qpath),
        _ => false,
    }
}

/// Check if a QPath contains "OperationBuilder"
fn path_contains_operation_builder(qpath: &rustc_hir::QPath<'_>) -> bool {
    match qpath {
        rustc_hir::QPath::Resolved(_, path) => path
            .segments
            .iter()
            .any(|seg| seg.ident.name.as_str() == "OperationBuilder"),
        rustc_hir::QPath::TypeRelative(ty, segment) => {
            segment.ident.name.as_str() == "OperationBuilder" || type_contains_operation_builder(ty)
        }
        _ => false,
    }
}

/// Recursively check if a type contains "OperationBuilder"
fn type_contains_operation_builder(ty: &rustc_hir::Ty<'_>) -> bool {
    match &ty.kind {
        rustc_hir::TyKind::Path(qpath) => path_contains_operation_builder(qpath),
        _ => false,
    }
}

/// Check if the first argument is an OData parameter and emit lint if so
fn check_odata_param<'tcx>(cx: &LateContext<'tcx>, arg: &'tcx Expr<'tcx>, method_name: &str) {
    if let ExprKind::Lit(lit) = &arg.kind
        && let rustc_ast::ast::LitKind::Str(sym, _) = lit.node
    {
        let param_name = sym.as_str();

        // Check if this is an OData parameter
        if ODATA_PARAMS.contains(&param_name) {
            let recommended = get_recommended_method(param_name);

            cx.span_lint(DE0802_USE_ODATA_EXT, arg.span, |diag| {
                diag.primary_message(format!(
                    "use OperationBuilderODataExt instead of .{}() for OData parameter `{}` (DE0802)",
                    method_name, param_name
                ));
                diag.help(format!("use {} instead", recommended));
                diag.note(
                    "type-safe OData methods provide compile-time validation and automatic OpenAPI schema generation",
                );
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
        lint_utils::test_comment_annotations_match_stderr(&ui_dir, "DE0802", "use OData ext");
    }
}
