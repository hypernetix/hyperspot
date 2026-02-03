//! License validation middleware.
//!
//! Validates that the tenant has all required license features for the endpoint.
//!
//! # Behavior
//!
//! - No license requirement: Pass through
//! - Client unavailable: Stub behavior (only BASE feature allowed)
//! - Client available:
//!   - Missing `SecurityContext`: 401 Unauthorized
//!   - All features enabled: Pass through
//!   - Feature missing: 403 Forbidden
//!   - License service error: 503 Service Unavailable

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use http::Method;

use license_enforcer_sdk::{LicenseEnforcerGatewayClient, LicenseFeatureID};
use modkit::api::{OperationSpec, Problem};
use modkit_security::SecurityContext;

/// Base platform feature - included in all licenses.
/// Used as fallback when license client is not available.
const BASE_FEATURE: &str = "gts.x.core.lic.feat.v1~x.core.global.base.v1";

type LicenseKey = (Method, String);

/// Type alias for the license enforcer gateway client.
pub type LicenseClient = Arc<dyn LicenseEnforcerGatewayClient>;

/// Map of route (method, path) to required license features.
/// Immutable after construction - built once at startup from operation specs.
#[derive(Clone)]
pub struct LicenseRequirementMap {
    requirements: Arc<HashMap<LicenseKey, Arc<[String]>>>,
}

impl LicenseRequirementMap {
    /// Build a license requirement map from operation specs.
    #[must_use]
    pub fn from_specs(specs: &[OperationSpec]) -> Self {
        let mut requirements = HashMap::new();

        for spec in specs {
            if let Some(req) = spec.license_requirement.as_ref() {
                requirements.insert(
                    (spec.method.clone(), spec.path.clone()),
                    Arc::from(req.license_names.as_slice()),
                );
            }
        }

        Self {
            requirements: Arc::new(requirements),
        }
    }

    fn get(&self, method: &Method, path: &str) -> Option<Arc<[String]>> {
        self.requirements
            .get(&(method.clone(), path.to_owned()))
            .cloned()
    }
}

/// State for license validation middleware.
#[derive(Clone)]
pub struct LicenseValidationState {
    /// Optional license client (None if `license_enforcer` is not registered)
    pub client: Option<LicenseClient>,
    /// Map of route requirements
    pub map: LicenseRequirementMap,
}

// Helper functions to reduce cognitive complexity

fn forbidden_stub_response(required: &[String]) -> Response {
    Problem::new(
        StatusCode::FORBIDDEN,
        "Forbidden",
        format!(
            "Endpoint requires unsupported license features '{required:?}'; only '{BASE_FEATURE}' is allowed",
        ),
    )
    .into_response()
}

fn unauthorized_response() -> Response {
    Problem::new(
        StatusCode::UNAUTHORIZED,
        "Unauthorized",
        "License check requires authentication",
    )
    .into_response()
}

fn forbidden_feature_response(feature_name: &str) -> Response {
    Problem::new(
        StatusCode::FORBIDDEN,
        "Forbidden",
        format!("License feature '{feature_name}' is not enabled"),
    )
    .into_response()
}

fn service_unavailable_response() -> Response {
    Problem::new(
        StatusCode::SERVICE_UNAVAILABLE,
        "Service Unavailable",
        "License validation temporarily unavailable",
    )
    .into_response()
}

/// Handle the case when no license client is available (stub behavior).
fn handle_no_client(method: &Method, path: &str, required: &[String]) -> Option<Response> {
    if required.iter().any(|r| r != BASE_FEATURE) {
        tracing::warn!(
            method = %method,
            path = %path,
            required = ?required,
            "License client not available, rejecting non-BASE feature requirements"
        );
        return Some(forbidden_stub_response(required));
    }
    None
}

/// Handle the case when client is available but no security context.
fn handle_no_context(method: &Method, path: &str) -> Response {
    tracing::warn!(
        method = %method,
        path = %path,
        "License check requires authentication but no security context found"
    );
    unauthorized_response()
}

/// Map license enforcer error to HTTP response.
fn map_license_error_to_response(
    error: &license_enforcer_sdk::LicenseEnforcerError,
    method: &Method,
    path: &str,
) -> Response {
    use license_enforcer_sdk::LicenseEnforcerError;

    match error {
        LicenseEnforcerError::Authorization { message } => {
            tracing::warn!(
                error = %message,
                method = %method,
                path = %path,
                "License check failed due to authorization error"
            );
            Problem::new(
                StatusCode::FORBIDDEN,
                "Forbidden",
                format!("License authorization failed: {message}"),
            )
            .into_response()
        }
        LicenseEnforcerError::MissingTenantScope => {
            tracing::warn!(
                method = %method,
                path = %path,
                "License check failed due to missing tenant scope"
            );
            unauthorized_response()
        }
        _ => {
            tracing::error!(
                error = ?error,
                method = %method,
                path = %path,
                "License check failed due to service error"
            );
            service_unavailable_response()
        }
    }
}

/// Perform actual license check with client and context.
async fn check_features(
    client: &LicenseClient,
    ctx: &SecurityContext,
    tenant_id: uuid::Uuid,
    method: &Method,
    path: &str,
    required: &[String],
) -> Option<Response> {
    match client.enabled_global_features(ctx, tenant_id).await {
        Ok(enabled) => {
            for feature_name in required {
                let feature_id = LicenseFeatureID::from(feature_name.as_str());
                if !enabled.contains(&feature_id) {
                    tracing::info!(
                        method = %method,
                        path = %path,
                        feature = %feature_name,
                        tenant_id = ?tenant_id,
                        "License feature not enabled for tenant"
                    );
                    return Some(forbidden_feature_response(feature_name));
                }
            }
            None
        }
        Err(ref e) => Some(map_license_error_to_response(e, method, path)),
    }
}

/// License validation middleware.
///
/// Checks that the tenant has all required license features for the endpoint.
///
/// # Arguments
///
/// * `state` - License validation state containing optional client and requirement map
/// * `req` - Incoming HTTP request
/// * `next` - Next middleware in the chain
///
/// # Returns
///
/// The response from the next middleware, or an error response if license validation fails.
pub async fn license_validation_middleware(
    state: LicenseValidationState,
    req: Request,
    next: Next,
) -> Response {
    let method = req.method().clone();
    let path = req
        .extensions()
        .get::<axum::extract::MatchedPath>()
        .map_or_else(|| req.uri().path().to_owned(), |p| p.as_str().to_owned());

    // If no license requirements for this route, pass through
    let Some(required) = state.map.get(&method, &path) else {
        return next.run(req).await;
    };

    // If no features required (empty list), pass through
    if required.is_empty() {
        return next.run(req).await;
    }

    // Get security context from extensions (injected by auth middleware)
    let security_context = req.extensions().get::<SecurityContext>().cloned();

    match (&state.client, &security_context) {
        (None, _) => {
            if let Some(response) = handle_no_client(&method, &path, &required) {
                return response;
            }
        }
        (Some(_), None) => {
            return handle_no_context(&method, &path);
        }
        (Some(client), Some(ctx)) => {
            // Extract explicit tenant ID from context
            let tenant_id = ctx.tenant_id();
            if tenant_id.is_nil() {
                tracing::warn!(
                    method = %method,
                    path = %path,
                    "License check requires tenant scope but context has no tenant ID"
                );
                return unauthorized_response();
            }

            if let Some(response) =
                check_features(client, ctx, tenant_id, &method, &path, &required).await
            {
                return response;
            }
        }
    }

    next.run(req).await
}
