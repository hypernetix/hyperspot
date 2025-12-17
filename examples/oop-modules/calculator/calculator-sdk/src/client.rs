//! gRPC client implementation of CalculatorClient
//!
//! Internal client used by `wire_client()`. Not exported from SDK.

use std::sync::Arc;
use anyhow::Result;
use async_trait::async_trait;
use tonic::transport::Channel;
use modkit_security::PolicyEngine;
use modkit_transport_grpc::attach_secctx;
use modkit_transport_grpc::client::{connect_with_retry, GrpcClientConfig};

use crate::api::{CalculatorClient, CalculatorError};
use crate::proto::calculator_service_client::CalculatorServiceClient;
use crate::proto::AddRequest;

/// gRPC client implementation of CalculatorClient
pub(crate) struct CalculatorGrpcClient {
    inner: CalculatorServiceClient<Channel>,
}

impl CalculatorGrpcClient {
    /// Connect to the CalculatorService using default configuration with retries.
    pub async fn connect(uri: impl Into<String>) -> Result<Self> {
        let cfg = GrpcClientConfig::new("calculator");
        let channel: Channel = connect_with_retry(uri, &cfg).await?;
        Ok(Self {
            inner: CalculatorServiceClient::new(channel),
        })
    }
}

#[async_trait]
impl CalculatorClient for CalculatorGrpcClient {
    async fn add(&self, pe: Arc<dyn PolicyEngine>, a: i64, b: i64) -> Result<i64, CalculatorError> {
        let mut client = self.inner.clone();

        // Build request with SecurityCtx in metadata
        let proto_req = AddRequest { a, b };
        let mut request = tonic::Request::new(proto_req);

        // Attach SecurityCtx to metadata
        attach_secctx(request.metadata_mut(), pe.context())
            .map_err(|e| CalculatorError::Internal(e.to_string()))?;

        // Make the gRPC call
        let response = client
            .add(request)
            .await
            .map_err(|status| match status.code() {
                tonic::Code::Unauthenticated => {
                    CalculatorError::Unauthorized(status.message().to_string())
                }
                _ => CalculatorError::Transport(status.message().to_string()),
            })?;

        Ok(response.into_inner().sum)
    }
}
