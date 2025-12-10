//! Calculator Module definition
//!
//! A trivial example gRPC service that performs addition.
//! This module demonstrates the OoP (out-of-process) module pattern.

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use modkit::context::ModuleCtx;
use modkit::contracts::{GrpcServiceModule, RegisterGrpcServiceFn};

use calculator_sdk::{CalculatorServiceServer, SERVICE_NAME};

use crate::api::grpc::CalculatorServiceImpl;
use crate::domain::Service;

/// Calculator module.
///
/// Exposes the accumulator service via gRPC through the grpc_hub.
#[modkit::module(
    name = "calculator",
    capabilities = [grpc]
)]
pub struct CalculatorModule;

impl Default for CalculatorModule {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl modkit::Module for CalculatorModule {
    async fn init(&self, ctx: &ModuleCtx) -> Result<()> {
        tracing::info!("Initializing calculator module");

        // Create domain service
        let service = Arc::new(Service::new());

        // Register Service in ClientHub for gRPC layer to use
        ctx.client_hub().register::<Service>(service);

        tracing::info!("calculator module initialized");
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Export gRPC services to grpc_hub
#[async_trait]
impl GrpcServiceModule for CalculatorModule {
    async fn get_grpc_services(&self, ctx: &ModuleCtx) -> Result<Vec<RegisterGrpcServiceFn>> {
        // Get Service from ClientHub
        let service = ctx
            .client_hub()
            .get::<Service>()
            .map_err(|e| anyhow::anyhow!("Service not available: {}", e))?;

        // Build CalculatorService with our domain service
        let svc = CalculatorServiceServer::new(CalculatorServiceImpl::new(service));

        Ok(vec![RegisterGrpcServiceFn {
            service_name: SERVICE_NAME,
            register: Box::new(move |routes| {
                routes.add_service(svc.clone());
            }),
        }])
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_service_add() {
        let service = Service::new();
        assert_eq!(service.add(10, 20), 30);
    }

    #[test]
    fn test_negative_numbers() {
        let service = Service::new();
        assert_eq!(service.add(-5, 3), -2);
    }
}
