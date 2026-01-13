//! Output ports (interfaces) for domain services.

use async_trait::async_trait;
use modkit_security::SecurityContext;
use oagw_sdk::Secret;
use uuid::Uuid;

use super::error::DomainError;

/// Port for resolving secrets from cred_store.
#[async_trait]
pub trait SecretResolver: Send + Sync {
    /// Get a secret by its reference UUID.
    async fn get_secret(
        &self,
        ctx: &SecurityContext,
        secret_ref: Uuid,
    ) -> Result<Secret, DomainError>;
}

/// Stub implementation of SecretResolver for v1.
///
/// TODO(v2): Replace with actual cred_store integration.
#[derive(Debug, Default)]
pub struct StubSecretResolver;

#[async_trait]
impl SecretResolver for StubSecretResolver {
    async fn get_secret(
        &self,
        _ctx: &SecurityContext,
        secret_ref: Uuid,
    ) -> Result<Secret, DomainError> {
        // TODO(v2): Implement actual cred_store lookup via CredStoreApi
        // For v1, return a stub secret for testing
        tracing::warn!(
            secret_ref = %secret_ref,
            "Using stub secret resolver - replace with cred_store integration"
        );

        Ok(Secret {
            id: secret_ref,
            secret_type_gts_id: "gts.x.core.oagw.auth_type.v1~x.core.auth.bearer_token.v1"
                .to_string(),
            value: "stub-secret-value".to_string(),
            metadata: None,
        })
    }
}
