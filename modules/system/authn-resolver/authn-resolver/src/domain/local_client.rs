//! Local (in-process) client for the `AuthN` resolver.

use std::sync::Arc;

use async_trait::async_trait;
use authn_resolver_sdk::{AuthNResolverClient, AuthNResolverError, AuthenticationResult};
use modkit_macros::domain_model;

use super::{DomainError, Service};

/// Local client wrapping the service.
///
/// Registered in `ClientHub` by the module during `init()`.
#[domain_model]
pub struct AuthNResolverLocalClient {
    svc: Arc<Service>,
}

impl AuthNResolverLocalClient {
    #[must_use]
    pub fn new(svc: Arc<Service>) -> Self {
        Self { svc }
    }
}

fn log_and_convert(op: &str, e: DomainError) -> AuthNResolverError {
    tracing::error!(operation = op, error = ?e, "authn_resolver call failed");
    e.into()
}

#[async_trait]
impl AuthNResolverClient for AuthNResolverLocalClient {
    async fn authenticate(
        &self,
        bearer_token: &str,
    ) -> Result<AuthenticationResult, AuthNResolverError> {
        self.svc
            .authenticate(bearer_token)
            .await
            .map_err(|e| log_and_convert("authenticate", e))
    }
}
