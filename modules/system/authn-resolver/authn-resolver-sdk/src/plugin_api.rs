//! Plugin API trait for `AuthN` resolver implementations.
//!
//! Plugins implement this trait to provide token validation.
//! The gateway discovers plugins via GTS types-registry and delegates
//! API calls to the selected plugin.

use async_trait::async_trait;

use crate::error::AuthNResolverError;
use crate::models::AuthenticationResult;

/// Plugin API trait for `AuthN` resolver implementations.
///
/// Each plugin registers this trait with a scoped `ClientHub` entry
/// using its GTS instance ID as the scope.
///
/// The gateway delegates to this method. Cross-cutting concerns (logging,
/// metrics) may be added at the gateway level in the future.
#[async_trait]
pub trait AuthNResolverPluginClient: Send + Sync {
    /// Authenticate a bearer token and return the validated identity.
    ///
    /// # Arguments
    ///
    /// * `bearer_token` - The raw bearer token string
    ///
    /// # Errors
    ///
    /// - `Unauthorized` if the token is invalid, expired, or malformed
    /// - `Internal` for unexpected errors
    async fn authenticate(
        &self,
        bearer_token: &str,
    ) -> Result<AuthenticationResult, AuthNResolverError>;
}
