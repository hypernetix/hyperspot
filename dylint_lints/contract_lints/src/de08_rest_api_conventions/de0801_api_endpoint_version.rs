//! DE0801: API Endpoint Must Have Version
//!
//! All API endpoints MUST include a major version in the path.
//! The version must be in the format `/v{N}/` where N is a major version number.
//!
//! ## Example: Bad
//!
//! ```rust,ignore
//! // src/api/rest/routes.rs - WRONG
//! use modkit::api::OperationBuilder;
//!
//! pub fn routes() -> Router {
//!     let router = Router::new();
//!
//!     // ❌ No version in path
//!     let router = OperationBuilder::get("/users")
//!         .handler(list_users)
//!         .build(router);
//!
//!     // ❌ Semver not allowed (major version only)
//!     let router = OperationBuilder::get("/v1.0/users/{id}")
//!         .handler(get_user)
//!         .build(router);
//!
//!     // ❌ No version segment
//!     let router = OperationBuilder::post("/users/{id}/activate")
//!         .handler(activate_user)
//!         .build(router);
//!
//!     router
//! }
//! ```
//!
//! ## Example: Good
//!
//! ```rust,ignore
//! // src/api/rest/routes.rs - CORRECT
//! use modkit::api::OperationBuilder;
//!
//! pub fn routes() -> Router {
//!     let router = Router::new();
//!
//!     // ✅ Version prefix pattern
//!     let router = OperationBuilder::get("/v1/users")
//!         .handler(list_users)
//!         .build(router);
//!
//!     // ✅ Version prefix with path params
//!     let router = OperationBuilder::get("/v1/users/{id}")
//!         .handler(get_user)
//!         .build(router);
//!
//!     // ✅ Version suffix pattern (alternative)
//!     let router = OperationBuilder::post("/users/v1")
//!         .handler(create_user)
//!         .build(router);
//!
//!     // ✅ Major version only (v2, v3, etc.)
//!     let router = OperationBuilder::put("/users/v2/{id}")
//!         .handler(update_user_v2)
//!         .build(router);
//!
//!     router
//! }
//! ```

use rustc_hir::{Expr, ExprKind};
use rustc_lint::{LateContext, LintContext};
use rustc_span::Span;

rustc_session::declare_lint! {
    /// DE0801: API endpoint must have version
    ///
    /// API endpoints must include a major version in the path (e.g., `/v1/nodes` or `/nodes/v1`).
    /// This ensures API versioning for backward compatibility.
    pub DE0801_API_ENDPOINT_MUST_HAVE_VERSION,
    Deny,
    "API endpoints must include a major version in the path (DE0801)"
}

/// Check if a path contains a valid version segment
fn has_valid_version(path: &str) -> bool {
    // Check for version prefix: /v{N}/...
    let has_prefix = {
        let mut chars = path.chars().peekable();
        if chars.next() == Some('/') && chars.next() == Some('v') {
            // Must have at least one digit
            let mut has_digit = false;
            while let Some(&c) = chars.peek() {
                if c.is_ascii_digit() {
                    has_digit = true;
                    chars.next();
                } else {
                    break;
                }
            }
            // Must be followed by '/' (not a dot for semver)
            has_digit && chars.next() == Some('/')
        } else {
            false
        }
    };

    if has_prefix {
        return true;
    }

    // Check for version suffix: .../v{N}
    if let Some(pos) = path.rfind("/v") {
        let after_v = &path[pos + 2..];
        // Must have at least one digit and nothing after (or only digits)
        let mut chars = after_v.chars().peekable();
        let mut has_digit = false;
        while let Some(c) = chars.next() {
            if c.is_ascii_digit() {
                has_digit = true;
            } else {
                // If there's anything after digits, it's not a valid suffix
                return false;
            }
        }
        return has_digit;
    }

    false
}

/// HTTP method names that OperationBuilder uses
const HTTP_METHODS: &[&str] = &["get", "post", "put", "delete", "patch"];

/// Recursively check if a type contains "OperationBuilder"
fn type_contains_operation_builder(ty: &rustc_hir::Ty<'_>) -> bool {
    match &ty.kind {
        rustc_hir::TyKind::Path(qpath) => match qpath {
            rustc_hir::QPath::Resolved(_, path) => path
                .segments
                .iter()
                .any(|seg| seg.ident.name.as_str() == "OperationBuilder"),
            rustc_hir::QPath::TypeRelative(inner_ty, segment) => {
                segment.ident.name.as_str() == "OperationBuilder"
                    || type_contains_operation_builder(inner_ty)
            }
            #[allow(unreachable_patterns)]
            _ => false,
        },
        _ => false,
    }
}

/// Check if an expression is an OperationBuilder method call and validate the path
pub fn check_expr<'tcx>(cx: &LateContext<'tcx>, expr: &'tcx Expr<'tcx>) {
    // Look for method calls like OperationBuilder::<...>::get("/path"), ::post("/path"), etc.
    if let ExprKind::Call(func, args) = &expr.kind {
        if let ExprKind::Path(qpath) = &func.kind {
            let is_operation_builder_http_method = match qpath {
                // Handle: OperationBuilder::<Missing, Missing, ()>::get
                // This creates a TypeRelative where the type is OperationBuilder::<...>
                // and the segment is "get"
                rustc_hir::QPath::TypeRelative(ty, segment) => {
                    let method_name = segment.ident.name.as_str();
                    let is_http_method = HTTP_METHODS.contains(&method_name);

                    if is_http_method {
                        // Check if the type contains OperationBuilder
                        type_contains_operation_builder(ty)
                    } else {
                        false
                    }
                }
                // Handle: OperationBuilder::get (without type params) - resolved path
                rustc_hir::QPath::Resolved(_, path) => {
                    let segments: Vec<&str> = path
                        .segments
                        .iter()
                        .map(|seg| seg.ident.name.as_str())
                        .collect();

                    // Check for pattern like ["OperationBuilder", "get"]
                    if segments.len() >= 2 {
                        let has_op_builder = segments.iter().any(|s| *s == "OperationBuilder");
                        let last_is_http_method = segments
                            .last()
                            .map(|s| HTTP_METHODS.contains(s))
                            .unwrap_or(false);
                        has_op_builder && last_is_http_method
                    } else {
                        false
                    }
                }
                #[allow(unreachable_patterns)]
                _ => false,
            };

            if is_operation_builder_http_method {
                // The first argument should be the path string
                if let Some(path_arg) = args.first() {
                    check_path_argument(cx, path_arg, expr.span);
                }
            }
        }
    }
}

fn check_path_argument<'tcx>(cx: &LateContext<'tcx>, path_arg: &'tcx Expr<'tcx>, span: Span) {
    // Extract string literal from the path argument
    if let ExprKind::Lit(lit) = &path_arg.kind {
        if let rustc_ast::ast::LitKind::Str(sym, _) = lit.node {
            let path = sym.as_str();

            // Skip if path already has a valid version
            if has_valid_version(path) {
                return;
            }

            // Report the lint
            cx.span_lint(DE0801_API_ENDPOINT_MUST_HAVE_VERSION, span, |diag| {
                diag.primary_message(format!(
                    "API endpoint `{}` is missing a version segment (DE0801)",
                    path
                ));
                diag.help(format!(
                    "add a major version to the path, e.g., `/v1{}` or `{}/v1`",
                    path, path
                ));
                diag.note("version must be major only (v1, v2, etc.), not semver (v1.0)");
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_detection() {
        // Valid patterns
        assert!(has_valid_version("/v1/nodes"));
        assert!(has_valid_version("/v2/users/{id}"));
        assert!(has_valid_version("/v10/resources"));
        assert!(has_valid_version("/nodes/v1"));
        assert!(has_valid_version("/users/{id}/v2"));
        assert!(has_valid_version("/v1/nodes/{id}/details"));

        // Invalid patterns
        assert!(!has_valid_version("/nodes"));
        assert!(!has_valid_version("/nodes/{id}"));
        assert!(!has_valid_version("/users/{id}/profile"));
        assert!(!has_valid_version("/v1.0/nodes")); // semver not allowed
        assert!(!has_valid_version("/v1.2.3/nodes")); // semver not allowed
    }
}
