#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;
extern crate rustc_hir;

use rustc_hir::{Expr, ExprKind};
use rustc_lint::{LateContext, LateLintPass, LintContext};

dylint_linting::declare_late_lint! {
    /// ### What it does
    ///
    /// Checks that API endpoints follow the format `/{service-name}/v{N}/{resource}`.
    ///
    /// ### Why is this bad?
    ///
    /// Consistent API structure ensures proper versioning and organization.
    /// Service names help identify different microservices/modules, and versions
    /// allow for API evolution without breaking changes.
    ///
    /// ### Example
    ///
    /// ```rust,ignore
    /// // Bad - no service name or version
    /// OperationBuilder::get("/users")
    ///
    /// // Bad - no service name before version
    /// OperationBuilder::get("/v1/users")
    ///
    /// // Bad - service name uses underscore
    /// OperationBuilder::post("/some_service/v1/users")
    /// ```
    ///
    /// Use instead:
    ///
    /// ```rust,ignore
    /// // Good - correct format
    /// OperationBuilder::get("/my-service/v1/users")
    ///
    /// // Good - with path parameters
    /// OperationBuilder::get("/my-service/v1/users/{id}")
    ///
    /// // Good - with sub-resources
    /// OperationBuilder::post("/my-service/v2/users/{id}/profile")
    /// ```
    pub DE0801_API_ENDPOINT_VERSION,
    Deny,
    "API endpoints must follow /{service-name}/v{N}/{resource} format (DE0801)"
}

impl<'tcx> LateLintPass<'tcx> for De0801ApiEndpointVersion {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'tcx>) {
        if let ExprKind::Call(func, args) = &expr.kind
            && let ExprKind::Path(qpath) = &func.kind
        {
            let is_operation_builder_http_method = match qpath {
                rustc_hir::QPath::TypeRelative(ty, segment) => {
                    let method_name = segment.ident.name.as_str();
                    let is_http_method = HTTP_METHODS.contains(&method_name);

                    if is_http_method {
                        type_contains_operation_builder(ty)
                    } else {
                        false
                    }
                }
                rustc_hir::QPath::Resolved(_, path) => {
                    let segments: Vec<&str> = path
                        .segments
                        .iter()
                        .map(|seg| seg.ident.name.as_str())
                        .collect();

                    if segments.len() >= 2 {
                        let has_op_builder = segments.contains(&"OperationBuilder");
                        let last_is_http_method = segments
                            .last()
                            .map(|s| HTTP_METHODS.contains(s))
                            .unwrap_or(false);
                        has_op_builder && last_is_http_method
                    } else {
                        false
                    }
                }
                _ => false,
            };

            if is_operation_builder_http_method && let Some(path_arg) = args.first() {
                check_path_argument(cx, path_arg);
            }
        }
    }
}

/// Result of path validation
#[derive(Debug, PartialEq)]
enum PathValidationError {
    /// No service name before version
    MissingServiceName,
    /// Service name is not in kebab-case
    InvalidServiceName(String),
    /// Missing version segment
    MissingVersion,
    /// Invalid version format (not v{N})
    InvalidVersionFormat(String),
    /// Missing resource after version
    MissingResource,
    /// Resource or sub-resource is not in kebab-case
    InvalidResourceName(String),
}

/// Check if a segment is a valid kebab-case identifier
fn is_valid_kebab_case(segment: &str) -> bool {
    if segment.is_empty() {
        return false;
    }

    if segment.starts_with('-') || segment.ends_with('-') {
        return false;
    }

    segment
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Check if a segment is a valid version (v{N})
fn is_valid_version(segment: &str) -> bool {
    if !segment.starts_with('v') {
        return false;
    }

    let after_v = &segment[1..];
    if after_v.is_empty() {
        return false;
    }

    after_v.chars().all(|c| c.is_ascii_digit())
}

/// Check if a segment is a path parameter like {id}
fn is_path_param(segment: &str) -> bool {
    segment.starts_with('{') && segment.ends_with('}')
}

/// Validate that a path follows the format: /{service-name}/v{N}/{resource}
fn validate_api_path(path: &str) -> Result<(), PathValidationError> {
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    if segments.is_empty() {
        return Err(PathValidationError::MissingServiceName);
    }

    // First segment must be service name (not a version)
    let service_name = segments[0];
    if is_valid_version(service_name) {
        return Err(PathValidationError::MissingServiceName);
    }

    if !is_valid_kebab_case(service_name) {
        return Err(PathValidationError::InvalidServiceName(
            service_name.to_string(),
        ));
    }

    // Second segment must be version
    if segments.len() < 2 {
        return Err(PathValidationError::MissingVersion);
    }
    let version = segments[1];
    if !is_valid_version(version) {
        return Err(PathValidationError::InvalidVersionFormat(
            version.to_string(),
        ));
    }

    // Must have at least one resource after version
    if segments.len() < 3 {
        return Err(PathValidationError::MissingResource);
    }

    // Validate all remaining segments (resources and sub-resources)
    for segment in &segments[2..] {
        if is_path_param(segment) {
            continue;
        }
        if !is_valid_kebab_case(segment) {
            return Err(PathValidationError::InvalidResourceName(
                (*segment).to_string(),
            ));
        }
    }

    Ok(())
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
            _ => false,
        },
        _ => false,
    }
}

fn check_path_argument<'tcx>(cx: &LateContext<'tcx>, path_arg: &'tcx Expr<'tcx>) {
    if let ExprKind::Lit(lit) = &path_arg.kind
        && let rustc_ast::ast::LitKind::Str(sym, _) = lit.node
    {
        let path = sym.as_str();

        if let Err(err) = validate_api_path(path) {
            let (message, help, note) = match err {
                PathValidationError::MissingServiceName => (
                    format!(
                        "API endpoint `{}` is missing a service name before version (DE0801)",
                        path
                    ),
                    "use format: /{service-name}/v{N}/{resource}".to_string(),
                    "service name must come before version segment".to_string(),
                ),
                PathValidationError::InvalidServiceName(name) => (
                    format!(
                        "API endpoint `{}` has invalid service name `{}` (DE0801)",
                        path, name
                    ),
                    "service name must be kebab-case (lowercase letters, numbers, dashes)"
                        .to_string(),
                    "service name must not start or end with a dash".to_string(),
                ),
                PathValidationError::MissingVersion => (
                    format!(
                        "API endpoint `{}` is missing a version segment (DE0801)",
                        path
                    ),
                    "add version as second segment: /{service-name}/v{N}/{resource}".to_string(),
                    "version must be v1, v2, etc.".to_string(),
                ),
                PathValidationError::InvalidVersionFormat(ver) => (
                    format!(
                        "API endpoint `{}` has invalid version format `{}` (DE0801)",
                        path, ver
                    ),
                    "version must be lowercase 'v' followed by digits (v1, v2, v10)".to_string(),
                    "semver (v1.0) and uppercase (V1) are not allowed".to_string(),
                ),
                PathValidationError::MissingResource => (
                    format!(
                        "API endpoint `{}` is missing a resource after version (DE0801)",
                        path
                    ),
                    "add resource: /{service-name}/v{N}/{resource}".to_string(),
                    "at least one resource segment is required after version".to_string(),
                ),
                PathValidationError::InvalidResourceName(name) => (
                    format!(
                        "API endpoint `{}` has invalid resource name `{}` (DE0801)",
                        path, name
                    ),
                    "resource names must be kebab-case (lowercase letters, numbers, dashes)"
                        .to_string(),
                    "resource names must not start or end with a dash".to_string(),
                ),
            };

            cx.span_lint(DE0801_API_ENDPOINT_VERSION, path_arg.span, |diag| {
                diag.primary_message(message);
                diag.help(help);
                diag.note(note);
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
            "DE0801",
            "API endpoint version",
        );
    }
}
