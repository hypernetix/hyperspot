//! Domain errors for OAGW.

use oagw_sdk::OagwError;
use thiserror::Error;
use uuid::Uuid;

/// Domain-level errors for OAGW operations.
#[derive(Error, Debug)]
pub enum DomainError {
    /// Route not found.
    #[error("route not found: {id}")]
    RouteNotFound { id: Uuid },

    /// Link not found.
    #[error("link not found: {id}")]
    LinkNotFound { id: Uuid },

    /// Link is unavailable (disabled or filtered out).
    #[error("no available link for route {route_id}")]
    LinkUnavailable { route_id: Uuid },

    /// Route already exists.
    #[error("route already exists: {id}")]
    RouteAlreadyExists { id: Uuid },

    /// Link already exists.
    #[error("link already exists: {id}")]
    LinkAlreadyExists { id: Uuid },

    /// Invalid route configuration.
    #[error("invalid route: {message}")]
    InvalidRoute { message: String },

    /// Invalid link configuration.
    #[error("invalid link: {message}")]
    InvalidLink { message: String },

    /// Plugin not found for requirements.
    #[error("no plugin found for protocol '{protocol}' and auth type '{auth_type}'")]
    PluginNotFound { protocol: String, auth_type: String },

    /// Plugin client not registered in ClientHub.
    #[error("plugin client not registered: {gts_id}")]
    PluginClientNotFound { gts_id: String },

    /// Secret not found in cred_store.
    #[error("secret not found: {secret_ref}")]
    SecretNotFound { secret_ref: Uuid },

    /// Types registry unavailable.
    #[error("types registry unavailable: {0}")]
    TypesRegistryUnavailable(String),

    /// Validation error.
    #[error("validation error: {field}: {message}")]
    Validation { field: String, message: String },

    /// Authorization error.
    #[error("forbidden: {message}")]
    Forbidden { message: String },

    /// Authorization scope preparation error.
    #[error("authorization error: {0}")]
    Authorization(String),

    /// Connection timeout.
    #[error("connection timeout")]
    ConnectionTimeout,

    /// Request timeout.
    #[error("request timeout")]
    RequestTimeout,

    /// Downstream error.
    #[error("downstream error: status {status_code}")]
    DownstreamError {
        status_code: u16,
        retry_after_sec: Option<u64>,
    },

    /// Database error.
    #[error("database error: {0}")]
    Database(#[from] anyhow::Error),
}

impl DomainError {
    /// Create a validation error.
    #[must_use]
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Create a forbidden error.
    #[must_use]
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::Forbidden {
            message: message.into(),
        }
    }
}

/// Convert DomainError to SDK OagwError.
impl From<DomainError> for OagwError {
    fn from(e: DomainError) -> Self {
        match e {
            DomainError::RouteNotFound { id } => Self::route_not_found(id),
            DomainError::LinkNotFound { id } => Self::link_not_found(id),
            DomainError::LinkUnavailable { route_id } => Self::link_unavailable(route_id),
            DomainError::RouteAlreadyExists { id } => {
                Self::validation(format!("route already exists: {id}"))
            }
            DomainError::LinkAlreadyExists { id } => {
                Self::validation(format!("link already exists: {id}"))
            }
            DomainError::InvalidRoute { message } | DomainError::InvalidLink { message } => {
                Self::validation(message)
            }
            DomainError::PluginNotFound {
                protocol,
                auth_type,
            } => Self::plugin_not_found(protocol, auth_type),
            DomainError::PluginClientNotFound { gts_id } => {
                Self::internal(format!("plugin client not registered: {gts_id}"))
            }
            DomainError::SecretNotFound { secret_ref } => Self::secret_not_found(secret_ref),
            DomainError::TypesRegistryUnavailable(msg) => Self::internal(msg),
            DomainError::Validation { field, message } => {
                Self::validation(format!("{field}: {message}"))
            }
            DomainError::Forbidden { message } => Self::forbidden(message),
            DomainError::Authorization(msg) => Self::forbidden(msg),
            DomainError::ConnectionTimeout => Self::ConnectionTimeout,
            DomainError::RequestTimeout => Self::RequestTimeout,
            DomainError::DownstreamError {
                status_code,
                retry_after_sec,
            } => Self::downstream_error(status_code, retry_after_sec),
            DomainError::Database(_) => Self::Database,
        }
    }
}
