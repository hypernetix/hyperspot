use crate::{claims::Claims, errors::AuthError, types::SecRequirement};
use async_trait::async_trait;
use modkit_security::SecurityContext;
use std::sync::Arc;

/// Validates and parses JWT tokens
#[async_trait]
pub trait TokenValidator: Send + Sync {
    /// Validate a JWT token and return normalized claims
    async fn validate_and_parse(&self, token: &str) -> Result<Claims, AuthError>;
}

/// Builds an `SecurityContext` from JWT claims
pub trait SecurityContextBuilder: Send + Sync {
    /// Build a security context from claims
    fn build(&self, claims: &Claims) -> SecurityContext;
}

/// Builds a policy engine from a security context
pub trait PolicyEngineBuilder: Send + Sync {
    /// Build a policy engine from a security context
    fn build(&self, context: &SecurityContext) -> Arc<dyn modkit_security::PolicyEngine>;
}

/// Primary authorizer that checks if claims satisfy a security requirement
#[async_trait]
pub trait PrimaryAuthorizer: Send + Sync {
    /// Check if the claims satisfy the required resource:action
    async fn check(&self, claims: &Claims, requirement: &SecRequirement) -> Result<(), AuthError>;
}
