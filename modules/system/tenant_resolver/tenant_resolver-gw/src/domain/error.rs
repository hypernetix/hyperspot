//! Domain errors for the tenant resolver gateway.

use hs_tenant_resolver_sdk::TenantResolverError;
use uuid::Uuid;

/// Internal domain errors.
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

    #[error("tenant not found: {tenant_id}")]
    TenantNotFound { tenant_id: Uuid },

    #[error("access denied to tenant: {target_tenant}")]
    AccessDenied { target_tenant: Uuid },

    #[error("unauthorized")]
    Unauthorized,

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

impl From<TenantResolverError> for DomainError {
    fn from(e: TenantResolverError) -> Self {
        match e {
            TenantResolverError::TenantNotFound { tenant_id } => Self::TenantNotFound { tenant_id },
            TenantResolverError::AccessDenied { target_tenant } => {
                Self::AccessDenied { target_tenant }
            }
            TenantResolverError::Unauthorized => Self::Unauthorized,
            TenantResolverError::NoPluginAvailable => Self::PluginNotFound {
                vendor: "unknown".to_owned(),
            },
            TenantResolverError::ServiceUnavailable(msg) => Self::PluginUnavailable {
                gts_id: "unknown".to_owned(),
                reason: msg,
            },
            TenantResolverError::Internal(msg) => Self::Internal(msg),
        }
    }
}

impl From<DomainError> for TenantResolverError {
    fn from(e: DomainError) -> Self {
        match e {
            DomainError::PluginNotFound { .. } => Self::NoPluginAvailable,
            DomainError::InvalidPluginInstance { gts_id, reason } => {
                Self::Internal(format!("invalid plugin instance '{gts_id}': {reason}"))
            }
            DomainError::PluginUnavailable { gts_id, reason } => {
                Self::ServiceUnavailable(format!("plugin not available for '{gts_id}': {reason}"))
            }
            DomainError::TenantNotFound { tenant_id } => Self::TenantNotFound { tenant_id },
            DomainError::AccessDenied { target_tenant } => Self::AccessDenied { target_tenant },
            DomainError::Unauthorized => Self::Unauthorized,
            DomainError::TypesRegistryUnavailable(reason) | DomainError::Internal(reason) => {
                Self::Internal(reason)
            }
        }
    }
}
