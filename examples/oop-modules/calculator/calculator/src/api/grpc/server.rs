//! gRPC Server implementation for calculator
//!
//! The server implementation handles gRPC requests and delegates
//! to the domain Service for business logic.

use std::sync::Arc;

use tonic::{Request, Response, Status};

use calculator_sdk::{AddRequest, AddResponse, CalculatorService};
use modkit_transport_grpc::extract_secctx;

use crate::domain::Service;

/// gRPC service implementation that wraps the domain Service.
#[derive(Clone)]
pub struct CalculatorServiceImpl {
    service: Arc<Service>,
}

impl CalculatorServiceImpl {
    /// Create a new CalculatorService implementation with the given Service.
    pub fn new(service: Arc<Service>) -> Self {
        Self { service }
    }
}

#[tonic::async_trait]
impl CalculatorService for CalculatorServiceImpl {
    async fn add(&self, request: Request<AddRequest>) -> Result<Response<AddResponse>, Status> {
        // Extract SecurityCtx from gRPC metadata (for authorization)
        let _ctx = extract_secctx(request.metadata())?;

        let req = request.into_inner();

        // Delegate to domain service
        let sum = self.service.add(req.a, req.b);

        Ok(Response::new(AddResponse { sum }))
    }
}
