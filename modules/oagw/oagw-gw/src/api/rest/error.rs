//! REST error mapping for OAGW.

use http::StatusCode;
use modkit::api::problem::Problem;

use crate::domain::error::DomainError;

/// Convert DomainError to Problem for REST responses.
impl From<DomainError> for Problem {
    fn from(e: DomainError) -> Self {
        let trace_id = tracing::Span::current()
            .id()
            .map(|id| id.into_u64().to_string());

        let (status, code, title, detail) = match &e {
            DomainError::RouteNotFound { id } => (
                StatusCode::NOT_FOUND,
                "OAGW_ROUTE_NOT_FOUND",
                "Route not found",
                format!("No route with id {id}"),
            ),
            DomainError::LinkNotFound { id } => (
                StatusCode::NOT_FOUND,
                "OAGW_LINK_NOT_FOUND",
                "Link not found",
                format!("No link with id {id}"),
            ),
            DomainError::LinkUnavailable { route_id } => (
                StatusCode::SERVICE_UNAVAILABLE,
                "OAGW_LINK_UNAVAILABLE",
                "No available link",
                format!("No available link for route {route_id}"),
            ),
            DomainError::RouteAlreadyExists { id } => (
                StatusCode::CONFLICT,
                "OAGW_ROUTE_EXISTS",
                "Route already exists",
                format!("Route with id {id} already exists"),
            ),
            DomainError::LinkAlreadyExists { id } => (
                StatusCode::CONFLICT,
                "OAGW_LINK_EXISTS",
                "Link already exists",
                format!("Link with id {id} already exists"),
            ),
            DomainError::InvalidRoute { message } => (
                StatusCode::BAD_REQUEST,
                "OAGW_INVALID_ROUTE",
                "Invalid route",
                message.clone(),
            ),
            DomainError::InvalidLink { message } => (
                StatusCode::BAD_REQUEST,
                "OAGW_INVALID_LINK",
                "Invalid link",
                message.clone(),
            ),
            DomainError::PluginNotFound {
                protocol,
                auth_type,
            } => (
                StatusCode::SERVICE_UNAVAILABLE,
                "OAGW_PLUGIN_NOT_FOUND",
                "No plugin available",
                format!("No plugin found for protocol '{protocol}' and auth type '{auth_type}'"),
            ),
            DomainError::PluginClientNotFound { gts_id } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "OAGW_PLUGIN_CLIENT_NOT_FOUND",
                "Plugin client not found",
                format!("Plugin client not registered: {gts_id}"),
            ),
            DomainError::SecretNotFound { secret_ref } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "OAGW_SECRET_NOT_FOUND",
                "Secret not found",
                format!("Secret {secret_ref} not found in credential store"),
            ),
            DomainError::TypesRegistryUnavailable(msg) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "OAGW_REGISTRY_UNAVAILABLE",
                "Types registry unavailable",
                msg.clone(),
            ),
            DomainError::Validation { field, message } => (
                StatusCode::BAD_REQUEST,
                "OAGW_VALIDATION",
                "Validation error",
                format!("{field}: {message}"),
            ),
            DomainError::Forbidden { message } => (
                StatusCode::FORBIDDEN,
                "OAGW_FORBIDDEN",
                "Forbidden",
                message.clone(),
            ),
            DomainError::Authorization(msg) => (
                StatusCode::FORBIDDEN,
                "OAGW_AUTHORIZATION",
                "Authorization error",
                msg.clone(),
            ),
            DomainError::ConnectionTimeout => (
                StatusCode::GATEWAY_TIMEOUT,
                "OAGW_CONNECTION_TIMEOUT",
                "Connection timeout",
                "Connection to downstream API timed out".to_string(),
            ),
            DomainError::RequestTimeout => (
                StatusCode::GATEWAY_TIMEOUT,
                "OAGW_REQUEST_TIMEOUT",
                "Request timeout",
                "Request to downstream API timed out".to_string(),
            ),
            DomainError::DownstreamError {
                status_code,
                retry_after_sec: _,
            } => (
                StatusCode::BAD_GATEWAY,
                "OAGW_DOWNSTREAM_ERROR",
                "Downstream error",
                format!("Downstream API returned status {status_code}"),
            ),
            DomainError::Database(_) => {
                tracing::error!(error = ?e, "Database error occurred");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "OAGW_INTERNAL",
                    "Internal Server Error",
                    "An internal error occurred".to_string(),
                )
            }
        };

        let mut problem = Problem::new(status, title, detail)
            .with_type(format!("https://errors.hyperspot.com/{code}"))
            .with_code(code);

        if let Some(id) = trace_id {
            problem = problem.with_trace_id(id);
        }

        problem
    }
}
