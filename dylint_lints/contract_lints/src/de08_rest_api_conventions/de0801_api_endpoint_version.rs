//! DE0801: API Endpoint Must Have Service name and Version
//!
//! All API endpoints MUST follow the format `/{service-name}/v{N}/{resource}`.
//!
//! Requirements:
//! - Service name must be in kebab-case (lowercase letters, numbers, dashes)
//! - Service name must not start or end with a dash
//! - Version must be lowercase `v` followed by major version number (v1, v2, etc.)
//! - Resource and sub-resources must be in kebab-case
//! - Path parameters like `{id}` are allowed
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
//!     // ❌ No service name or version
//!     let router = OperationBuilder::get("/users")
//!         .handler(list_users)
//!         .build(router);
//!
//!     // ❌ No service name before version
//!     let router = OperationBuilder::get("/v1/users/{id}")
//!         .handler(get_user)
//!         .build(router);
//!
//!     // ❌ Service name uses underscore instead of kebab-case
//!     let router = OperationBuilder::post("/some_service/v1/users")
//!         .handler(create_user)
//!         .build(router);
//!
//!     // ❌ Uppercase letters in service name
//!     let router = OperationBuilder::get("/SomeService/v1/users")
//!         .handler(list_users)
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
//!     // ✅ Correct format: /{service-name}/v{N}/{resource}
//!     let router = OperationBuilder::get("/my-service/v1/users")
//!         .handler(list_users)
//!         .build(router);
//!
//!     // ✅ With path parameters
//!     let router = OperationBuilder::get("/my-service/v1/users/{id}")
//!         .handler(get_user)
//!         .build(router);
//!
//!     // ✅ With sub-resources
//!     let router = OperationBuilder::post("/my-service/v2/users/{id}/profile")
//!         .handler(update_profile)
//!         .build(router);
//!
//!     router
//! }
//! ```

use rustc_hir::{Expr, ExprKind};
use rustc_lint::{LateContext, LintContext};
use rustc_span::Span;

rustc_session::declare_lint! {
    /// DE0801: API endpoint must have service name and version
    ///
    /// API endpoints must follow the format `/{service-name}/v{N}/{resource}`.
    /// This ensures consistent API structure and versioning.
    pub DE0801_API_ENDPOINT_MUST_HAVE_VERSION,
    Deny,
    "API endpoints must follow /{service-name}/v{N}/{resource} format (DE0801)"
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
/// Allows: lowercase letters, digits, dashes (not at start/end)
fn is_valid_kebab_case(segment: &str) -> bool {
    if segment.is_empty() {
        return false;
    }

    // Must not start or end with a dash
    if segment.starts_with('-') || segment.ends_with('-') {
        return false;
    }

    // All characters must be lowercase letters, digits, or dashes
    segment
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Check if a segment is a valid version (v{N} where N is one or more digits)
fn is_valid_version(segment: &str) -> bool {
    if !segment.starts_with('v') {
        return false;
    }

    let after_v = &segment[1..];
    if after_v.is_empty() {
        return false;
    }

    // Must be all digits (no dots for semver)
    after_v.chars().all(|c| c.is_ascii_digit())
}

/// Check if a segment is a path parameter like {id}
fn is_path_param(segment: &str) -> bool {
    segment.starts_with('{') && segment.ends_with('}')
}

/// Validate that a path follows the format: /{service-name}/v{N}/{resource}[/{sub-resource}]*
fn validate_api_path(path: &str) -> Result<(), PathValidationError> {
    // Split path into segments (filter out empty segments from leading/trailing slashes)
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    if segments.is_empty() {
        return Err(PathValidationError::MissingServiceName);
    }

    // First segment must be service name (kebab-case, NOT a version)
    let service_name = segments[0];

    // Check if first segment looks like a version - means service name is missing
    if is_valid_version(service_name) {
        return Err(PathValidationError::MissingServiceName);
    }

    // Service name must be valid kebab-case
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
        // Path parameters like {id} are allowed
        if is_path_param(segment) {
            continue;
        }
        // Otherwise must be valid kebab-case
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

            // Validate the path format
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
                        format!(
                            "service name must be kebab-case (lowercase letters, numbers, dashes)"
                        ),
                        "service name must not start or end with a dash".to_string(),
                    ),
                    PathValidationError::MissingVersion => (
                        format!("API endpoint `{}` is missing a version segment (DE0801)", path),
                        "add version as second segment: /{service-name}/v{N}/{resource}".to_string(),
                        "version must be v1, v2, etc.".to_string(),
                    ),
                    PathValidationError::InvalidVersionFormat(ver) => (
                        format!(
                            "API endpoint `{}` has invalid version format `{}` (DE0801)",
                            path, ver
                        ),
                        "version must be lowercase 'v' followed by digits (v1, v2, v10)"
                            .to_string(),
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

                cx.span_lint(DE0801_API_ENDPOINT_MUST_HAVE_VERSION, span, |diag| {
                    diag.primary_message(message);
                    diag.help(help);
                    diag.note(note);
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_api_paths() {
        // Valid patterns: /{service-name}/v{N}/{resource}
        assert!(validate_api_path("/tests/v1/users").is_ok());
        assert!(validate_api_path("/abc/v2/products").is_ok());
        assert!(validate_api_path("/a-b-c/v1/orders").is_ok());
        assert!(validate_api_path("/tests/v1/users/{id}").is_ok());
        assert!(validate_api_path("/tests/v2/users/{id}/update").is_ok());
        assert!(validate_api_path("/tests/v3/products/{id}").is_ok());
        assert!(validate_api_path("/my-service/v10/resources").is_ok());
        assert!(validate_api_path("/service1/v1/items/{id}/details").is_ok());
    }

    #[test]
    fn test_missing_service_name() {
        // Missing service name (version first)
        assert_eq!(
            validate_api_path("/v1/products"),
            Err(PathValidationError::MissingServiceName)
        );
    }

    #[test]
    fn test_missing_version() {
        // Only service name, no version (missing second segment)
        assert_eq!(
            validate_api_path("/users"),
            Err(PathValidationError::MissingVersion)
        );
        // Second segment is not a valid version format
        assert_eq!(
            validate_api_path("/users/{id}"),
            Err(PathValidationError::InvalidVersionFormat("{id}".to_string()))
        );
        assert_eq!(
            validate_api_path("/users/{id}/activate"),
            Err(PathValidationError::InvalidVersionFormat("{id}".to_string()))
        );
        assert_eq!(
            validate_api_path("/api/users"),
            Err(PathValidationError::InvalidVersionFormat("users".to_string()))
        );
    }

    #[test]
    fn test_invalid_version_format() {
        // Invalid version formats
        assert_eq!(
            validate_api_path("/version1/users"),
            Err(PathValidationError::InvalidVersionFormat(
                "users".to_string()
            ))
        );
        assert_eq!(
            validate_api_path("/some-service/V1/products"),
            Err(PathValidationError::InvalidVersionFormat("V1".to_string()))
        );
    }

    #[test]
    fn test_invalid_service_name() {
        // Service name not in kebab-case
        assert_eq!(
            validate_api_path("/some_service/v1/products"),
            Err(PathValidationError::InvalidServiceName(
                "some_service".to_string()
            ))
        );
        assert_eq!(
            validate_api_path("/SomeService/v1/products"),
            Err(PathValidationError::InvalidServiceName(
                "SomeService".to_string()
            ))
        );
        // Leading dash in service name
        assert_eq!(
            validate_api_path("/-some-service/v1/products"),
            Err(PathValidationError::InvalidServiceName(
                "-some-service".to_string()
            ))
        );
    }

    #[test]
    fn test_invalid_resource_name() {
        // Resource name has uppercase
        assert_eq!(
            validate_api_path("/some-service/v1/Products"),
            Err(PathValidationError::InvalidResourceName(
                "Products".to_string()
            ))
        );
        // Leading dash in resource name
        assert_eq!(
            validate_api_path("/some-service/v1/-products"),
            Err(PathValidationError::InvalidResourceName(
                "-products".to_string()
            ))
        );
        // Leading dash in sub-resource name
        assert_eq!(
            validate_api_path("/some-service/v1/products/-abc"),
            Err(PathValidationError::InvalidResourceName("-abc".to_string()))
        );
    }

    #[test]
    fn test_kebab_case_validation() {
        // Valid kebab-case
        assert!(is_valid_kebab_case("hello"));
        assert!(is_valid_kebab_case("hello-world"));
        assert!(is_valid_kebab_case("my-service-123"));
        assert!(is_valid_kebab_case("a"));
        assert!(is_valid_kebab_case("123"));

        // Invalid kebab-case
        assert!(!is_valid_kebab_case("")); // empty
        assert!(!is_valid_kebab_case("-hello")); // leading dash
        assert!(!is_valid_kebab_case("hello-")); // trailing dash
        assert!(!is_valid_kebab_case("Hello")); // uppercase
        assert!(!is_valid_kebab_case("hello_world")); // underscore
    }

    #[test]
    fn test_version_validation() {
        // Valid versions
        assert!(is_valid_version("v1"));
        assert!(is_valid_version("v2"));
        assert!(is_valid_version("v10"));
        assert!(is_valid_version("v123"));

        // Invalid versions
        assert!(!is_valid_version("V1")); // uppercase
        assert!(!is_valid_version("v")); // no digits
        assert!(!is_valid_version("version1")); // wrong format
        assert!(!is_valid_version("1")); // no 'v' prefix
        assert!(!is_valid_version("v1.0")); // semver
    }
}
