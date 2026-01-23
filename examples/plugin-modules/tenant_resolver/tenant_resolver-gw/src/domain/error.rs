/// Domain error for the `tenant_resolver` gateway example.
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

    #[error("tenant not found: {0}")]
    TenantNotFound(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl From<tenant_resolver_example_sdk::TenantResolverError> for DomainError {
    fn from(e: tenant_resolver_example_sdk::TenantResolverError) -> Self {
        use tenant_resolver_example_sdk::TenantResolverError;
        match e {
            TenantResolverError::NotFound(msg) => Self::TenantNotFound(msg),
            // Unauthorized maps to PermissionDenied since this is a gateway
            // and authentication is handled at the gateway layer.
            TenantResolverError::PermissionDenied(msg) | TenantResolverError::Unauthorized(msg) => {
                Self::PermissionDenied(msg)
            }
            TenantResolverError::ServiceUnavailable(msg) => Self::PluginUnavailable {
                gts_id: "unknown".to_owned(),
                reason: msg,
            },
            TenantResolverError::Internal(msg) => Self::Internal(msg),
        }
    }
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

impl From<DomainError> for tenant_resolver_example_sdk::TenantResolverError {
    fn from(e: DomainError) -> Self {
        match e {
            DomainError::PluginNotFound { vendor } => {
                Self::NotFound(format!("no plugin instances found for vendor '{vendor}'"))
            }
            DomainError::InvalidPluginInstance { gts_id, reason } => {
                Self::Internal(format!("invalid plugin instance '{gts_id}': {reason}"))
            }
            DomainError::PluginUnavailable { gts_id, reason } => {
                Self::ServiceUnavailable(format!("plugin not available for '{gts_id}': {reason}"))
            }
            DomainError::TenantNotFound(msg) => Self::NotFound(msg),
            DomainError::PermissionDenied(msg) => Self::PermissionDenied(msg),
            DomainError::TypesRegistryUnavailable(reason) | DomainError::Internal(reason) => {
                Self::Internal(reason)
            }
        }
    }
}
