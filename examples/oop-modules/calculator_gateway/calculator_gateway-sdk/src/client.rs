//! Local client implementation of CalculatorGatewayClient
//!
//! Internal client used by `wire_client()`. Not exported from SDK.

use std::sync::Arc;

use async_trait::async_trait;
use modkit_security::SecurityContext;

use calculator_gateway::{Service, ServiceError};

use crate::api::{CalculatorGatewayClient, CalculatorGatewayError};

/// Local client implementation that delegates to the module's Service.
pub(crate) struct CalculatorGatewayLocalClient {
    service: Arc<Service>,
}

impl CalculatorGatewayLocalClient {
    /// Create a new local client wrapping the Service.
    pub(crate) fn new(service: Arc<Service>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl CalculatorGatewayClient for CalculatorGatewayLocalClient {
    async fn add(
        &self,
        ctx: &SecurityContext,
        a: i64,
        b: i64,
    ) -> Result<i64, CalculatorGatewayError> {
        self.service.add(ctx, a, b).await.map_err(convert_error)
    }
}

/// Convert internal ServiceError to public CalculatorGatewayError
fn convert_error(err: ServiceError) -> CalculatorGatewayError {
    match err {
        ServiceError::RemoteError(msg) => CalculatorGatewayError::RemoteError(msg),
        ServiceError::Internal(msg) => CalculatorGatewayError::Internal(msg),
    }
}
