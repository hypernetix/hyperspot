//! Domain errors for the `AuthN` resolver.

use authn_resolver_sdk::AuthNResolverError;
use modkit_macros::domain_model;

/// Internal domain errors.
#[domain_model]
#[derive(thiserror::Error, Debug)]
pub enum DomainError {
    #[error("types registry is not available: {0}")]
    TypesRegistryUnavailable(String),

    #[error("no plugin instances found for vendor '{vendor}'")]
    PluginNotFound { vendor: String },

    #[error("invalid plugin instance content for '{gts_id}': {reason}")]
    InvalidPluginInstance { gts_id: String, reason: String },

    #[error("plugin not available for '{gts_id}': {reason}")]
    PluginUnavailable { gts_id: String, reason: String },

    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl From<types_registry_sdk::TypesRegistryError> for DomainError {
    fn from(e: types_registry_sdk::TypesRegistryError) -> Self {
        Self::Internal(e.to_string())
    }
}

impl From<modkit::client_hub::ClientHubError> for DomainError {
    fn from(e: modkit::client_hub::ClientHubError) -> Self {
        Self::Internal(e.to_string())
    }
}

impl From<serde_json::Error> for DomainError {
    fn from(e: serde_json::Error) -> Self {
        Self::Internal(e.to_string())
    }
}

impl From<AuthNResolverError> for DomainError {
    fn from(e: AuthNResolverError) -> Self {
        match e {
            AuthNResolverError::Unauthorized(msg) => Self::Unauthorized(msg),
            AuthNResolverError::NoPluginAvailable => Self::PluginNotFound {
                vendor: "unknown".to_owned(),
            },
            AuthNResolverError::ServiceUnavailable(msg) => Self::PluginUnavailable {
                gts_id: "unknown".to_owned(),
                reason: msg,
            },
            AuthNResolverError::Internal(msg) => Self::Internal(msg),
        }
    }
}

impl From<DomainError> for AuthNResolverError {
    fn from(e: DomainError) -> Self {
        match e {
            DomainError::PluginNotFound { .. } => Self::NoPluginAvailable,
            DomainError::InvalidPluginInstance { gts_id, reason } => {
                Self::Internal(format!("invalid plugin instance '{gts_id}': {reason}"))
            }
            DomainError::PluginUnavailable { gts_id, reason } => {
                Self::ServiceUnavailable(format!("plugin not available for '{gts_id}': {reason}"))
            }
            DomainError::Unauthorized(msg) => Self::Unauthorized(msg),
            DomainError::TypesRegistryUnavailable(reason) | DomainError::Internal(reason) => {
                Self::Internal(reason)
            }
        }
    }
}
