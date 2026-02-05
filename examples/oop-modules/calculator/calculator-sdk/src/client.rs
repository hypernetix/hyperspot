//! gRPC client implementation of CalculatorClientV1
//!
//! Internal client used by `wire_client()`. Not exported from SDK.

use anyhow::Result;
use async_trait::async_trait;
use tonic::transport::Channel;

use modkit_security::SecurityContext;
use modkit_transport_grpc::attach_secctx;
use modkit_transport_grpc::client::{GrpcClientConfig, connect_with_retry};

use crate::api::{CalculatorClientV1, CalculatorError};
use crate::proto::AddRequest;
use crate::proto::calculator_service_client::CalculatorServiceClient;

/// gRPC client implementation of CalculatorClientV1
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
impl CalculatorClientV1 for CalculatorGrpcClient {
    async fn add(&self, ctx: &SecurityContext, a: i64, b: i64) -> Result<i64, CalculatorError> {
        let mut client = self.inner.clone();

        // Build request with SecurityContext in metadata
        let proto_req = AddRequest { a, b };
        let mut request = tonic::Request::new(proto_req);

        // Attach SecurityContext to metadata
        attach_secctx(request.metadata_mut(), ctx)
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
