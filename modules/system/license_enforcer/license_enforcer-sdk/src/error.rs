//! Error types for license enforcement.

use thiserror::Error;

/// Errors that can occur during license enforcement operations.
#[derive(Debug, Error)]
pub enum LicenseEnforcerError {
    /// License not found for the specified tenant/feature
    #[error("License not found for tenant {tenant_id}: {feature}")]
    LicenseNotFound {
        /// Tenant ID
        tenant_id: String,
        /// Feature identifier
        feature: String,
    },

    /// License has expired
    #[error("License expired for feature {feature}")]
    LicenseExpired {
        /// Feature identifier
        feature: String,
    },

    /// License is suspended
    #[error("License suspended for feature {feature}")]
    LicenseSuspended {
        /// Feature identifier
        feature: String,
    },

    /// Invalid license data
    #[error("Invalid license data: {reason}")]
    InvalidLicense {
        /// Reason for invalidity
        reason: String,
    },

    /// Platform integration error
    #[error("Platform integration error: {message}")]
    PlatformError {
        /// Error message
        message: String,
        /// Source error from platform integration
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Cache operation error
    #[error("Cache operation error: {message}")]
    CacheError {
        /// Error message
        message: String,
        /// Source error from cache operation
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Internal error
    #[error("Internal error: {message}")]
    Internal {
        /// Error message
        message: String,
        /// Source error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl LicenseEnforcerError {
    /// Create a platform error with a message only.
    pub fn platform(message: impl Into<String>) -> Self {
        Self::PlatformError {
            message: message.into(),
            source: None,
        }
    }

    /// Create a platform error with a source error.
    pub fn platform_with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::PlatformError {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a cache error with a message only.
    pub fn cache(message: impl Into<String>) -> Self {
        Self::CacheError {
            message: message.into(),
            source: None,
        }
    }

    /// Create a cache error with a source error.
    pub fn cache_with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::CacheError {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create an internal error with a message only.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
            source: None,
        }
    }

    /// Create an internal error with a source error.
    pub fn internal_with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::Internal {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }
}
