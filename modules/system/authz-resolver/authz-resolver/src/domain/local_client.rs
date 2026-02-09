//! Local (in-process) client for the `AuthZ` resolver.

use std::sync::Arc;

use async_trait::async_trait;
use authz_resolver_sdk::{
    AuthZResolverClient, AuthZResolverError, EvaluationRequest, EvaluationResponse,
};
use modkit_macros::domain_model;

use super::{DomainError, Service};

/// Local client wrapping the service.
#[domain_model]
pub struct AuthZResolverLocalClient {
    svc: Arc<Service>,
}

impl AuthZResolverLocalClient {
    #[must_use]
    pub fn new(svc: Arc<Service>) -> Self {
        Self { svc }
    }
}

fn log_and_convert(op: &str, e: DomainError) -> AuthZResolverError {
    tracing::error!(operation = op, error = ?e, "authz_resolver call failed");
    e.into()
}

#[async_trait]
impl AuthZResolverClient for AuthZResolverLocalClient {
    async fn evaluate(
        &self,
        request: EvaluationRequest,
    ) -> Result<EvaluationResponse, AuthZResolverError> {
        self.svc
            .evaluate(request)
            .await
            .map_err(|e| log_and_convert("evaluate", e))
    }
}
