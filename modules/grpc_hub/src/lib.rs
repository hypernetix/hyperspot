//! gRPC Hub Module
//!
//! This module acts as the single gRPC server hub in the process.
//! It collects all gRPC service registrations from modules with the `grpc` capability
//! and serves them on a single port.

use async_trait::async_trait;
use modkit::{
    context::ModuleCtx,
    contracts::{GrpcHubModule, GrpcServiceHandle, Module},
    lifecycle::ReadySignal,
};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

/// The gRPC Hub module.
///
/// This module is declared with capabilities: [stateful, system, grpc_hub].
/// - `system`: ensures it initializes with high priority
/// - `grpc_hub`: marks it as the single gRPC hub in the process
/// - `stateful`: enables lifecycle management
#[derive(Default)]
#[modkit::module(
    name = "grpc_hub",
    capabilities = [stateful, system, grpc_hub],
    lifecycle(entry = "serve", await_ready)
)]
pub struct GrpcHub {
    // Future: store config for port, TLS settings, etc.
}

#[async_trait]
impl Module for GrpcHub {
    async fn init(&self, _ctx: &ModuleCtx) -> anyhow::Result<()> {
        tracing::info!("gRPC hub module initialized");
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[async_trait]
impl GrpcHubModule for GrpcHub {
    async fn run_grpc_host(
        self: Arc<Self>,
        _ctx: &ModuleCtx,
        services: Vec<GrpcServiceHandle>,
    ) -> anyhow::Result<()> {
        // For now, implement a minimal version that logs the received services.
        // In a real implementation, this would:
        // 1. Extract tonic service instances from the opaque handles
        // 2. Build a tonic::Server with all services
        // 3. Bind to a configured port
        // 4. Run the server with graceful shutdown via the cancellation token

        tracing::info!(
            count = services.len(),
            services = ?services.iter().map(|s| (s.module_name, s.service_name)).collect::<Vec<_>>(),
            "gRPC hub received services to register"
        );

        // TODO: In future iterations, build and run the actual tonic server here.
        // Example structure:
        // let mut server = tonic::transport::Server::builder();
        // for handle in services {
        //     // Downcast handle.inner to the actual service type
        //     // server = server.add_service(service);
        // }
        // server.serve(addr).await?;

        Ok(())
    }
}

impl GrpcHub {
    /// Lifecycle entry point for the module.
    ///
    /// This is called by the stateful lifecycle system.
    /// The actual server is started in `run_grpc_host`, which is called
    /// by the runtime during the gRPC registration phase.
    async fn serve(
        self: Arc<Self>,
        _cancel: CancellationToken,
        ready: ReadySignal,
    ) -> anyhow::Result<()> {
        // The gRPC server is started in run_grpc_host, not here.
        // This lifecycle entry is just for compatibility with the stateful module system.
        tracing::debug!("gRPC hub lifecycle entry called");
        ready.notify();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use modkit::{
        client_hub::ClientHub,
        context::{ConfigProvider, ModuleCtx},
        contracts::GrpcServiceHandle,
        registry::ModuleRegistry,
    };
    use std::sync::Arc;

    struct EmptyConfigProvider;
    impl ConfigProvider for EmptyConfigProvider {
        fn get_module_config(&self, _module_name: &str) -> Option<&serde_json::Value> {
            None
        }
    }

    fn test_module_ctx(cancel: CancellationToken) -> ModuleCtx {
        ModuleCtx::new(
            "test",
            Arc::new(EmptyConfigProvider),
            Arc::new(ClientHub::default()),
            cancel,
            None,
        )
    }

    #[tokio::test]
    async fn test_grpc_hub_is_discoverable() {
        let registry = ModuleRegistry::discover_and_build().expect("registry build failed");

        // Check if the grpc_hub module is registered
        let has_grpc_hub = registry
            .modules()
            .iter()
            .any(|e| e.name == "grpc_hub" && e.is_system && e.grpc_hub.is_some());

        assert!(
            has_grpc_hub,
            "grpc_hub module should be registered with system and grpc_hub capabilities"
        );
    }

    #[tokio::test]
    async fn test_grpc_hub_run_grpc_host() {
        let hub = Arc::new(GrpcHub::default());
        let ctx = test_module_ctx(CancellationToken::new());

        // Create some dummy service handles
        let handles = vec![
            GrpcServiceHandle {
                module_name: "test_service_1",
                service_name: "TestService1",
                inner: Arc::new("dummy_service_1"),
            },
            GrpcServiceHandle {
                module_name: "test_service_2",
                service_name: "TestService2",
                inner: Arc::new("dummy_service_2"),
            },
        ];

        // Call run_grpc_host and ensure it doesn't panic
        let result = hub.run_grpc_host(&ctx, handles).await;
        assert!(result.is_ok(), "run_grpc_host should succeed");
    }

    #[tokio::test]
    async fn test_grpc_hub_empty_services() {
        let hub = Arc::new(GrpcHub::default());
        let ctx = test_module_ctx(CancellationToken::new());

        // Call with no services
        let result = hub.run_grpc_host(&ctx, vec![]).await;
        assert!(
            result.is_ok(),
            "run_grpc_host should succeed with empty services"
        );
    }
}
