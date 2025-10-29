use crate::{claims::Claims, errors::AuthError, types::SecRequirement};
use async_trait::async_trait;
use modkit_security::AccessScope;

/// Builds an AccessScope from JWT claims
pub trait ScopeBuilder: Send + Sync {
    /// Convert tenant claims into an AccessScope
    fn tenants_to_scope(&self, claims: &Claims) -> AccessScope;
}

/// Primary authorizer that checks if claims satisfy a security requirement
#[async_trait]
pub trait PrimaryAuthorizer: Send + Sync {
    /// Check if the claims satisfy the required resource:action
    async fn check(&self, claims: &Claims, requirement: &SecRequirement) -> Result<(), AuthError>;
}
