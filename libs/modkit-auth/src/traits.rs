use crate::{claims::Claims, errors::AuthError, types::SecRequirement};
use async_trait::async_trait;

/// Validates and parses JWT tokens
#[async_trait]
pub trait TokenValidator: Send + Sync {
    /// Validate a JWT token and return normalized claims
    async fn validate_and_parse(&self, token: &str) -> Result<Claims, AuthError>;
}

/// Primary authorizer that checks if claims satisfy a security requirement
#[async_trait]
pub trait PrimaryAuthorizer: Send + Sync {
    /// Check if the claims satisfy the required resource:action
    async fn check(&self, claims: &Claims, requirement: &SecRequirement) -> Result<(), AuthError>;
}
