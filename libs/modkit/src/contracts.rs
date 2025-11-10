use async_trait::async_trait;
use axum::Router;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

pub use crate::api::OpenApiRegistry;

/// Core module: DI/wiring; do not rely on migrated schema here.
#[async_trait]
pub trait Module: Send + Sync + 'static {
    async fn init(&self, ctx: &crate::context::ModuleCtx) -> anyhow::Result<()>;
    fn as_any(&self) -> &dyn std::any::Any;
}

#[async_trait]
pub trait DbModule: Send + Sync {
    /// Runs AFTER init, BEFORE REST/start.
    async fn migrate(&self, db: &modkit_db::DbHandle) -> anyhow::Result<()>;
}

/// Pure wiring; must be sync. Runs AFTER DB migrations.
pub trait RestfulModule: Send + Sync {
    fn register_rest(
        &self,
        ctx: &crate::context::ModuleCtx,
        router: Router,
        openapi: &dyn OpenApiRegistry,
    ) -> anyhow::Result<Router>;
}

/// REST host module: handles ingress hosting with prepare/finalize phases.
/// Must be sync. Runs during REST phase, but doesn't start the server.
#[allow(dead_code)]
pub trait RestHostModule: Send + Sync + 'static {
    /// Prepare a base Router (e.g., global middlewares, /healthz) and optionally touch OpenAPI meta.
    /// Do NOT start the server here.
    fn rest_prepare(
        &self,
        ctx: &crate::context::ModuleCtx,
        router: Router,
    ) -> anyhow::Result<Router>;

    /// Finalize before start: attach /openapi.json, /docs, persist the Router internally if needed.
    /// Do NOT start the server here.
    fn rest_finalize(
        &self,
        ctx: &crate::context::ModuleCtx,
        router: Router,
    ) -> anyhow::Result<Router>;

    // Return OpenAPI registry of the module, e.g., to register endpoints
    fn as_registry(&self) -> &dyn crate::contracts::OpenApiRegistry;
}

#[async_trait]
pub trait StatefulModule: Send + Sync {
    async fn start(&self, cancel: CancellationToken) -> anyhow::Result<()>;
    async fn stop(&self, cancel: CancellationToken) -> anyhow::Result<()>;
}

/// Opaque handle for a gRPC service registration.
///
/// Core modkit does not know the actual tonic type; modules and grpc_hub downcast as needed.
/// This keeps the core transport-agnostic.
pub struct GrpcServiceHandle {
    pub module_name: &'static str,
    pub service_name: &'static str,
    pub inner: Arc<dyn std::any::Any + Send + Sync>,
}

/// Trait for modules that export gRPC services.
///
/// The runtime will call this during the gRPC registration phase to collect
/// all services that should be exposed on the shared gRPC server.
#[async_trait]
pub trait GrpcServiceModule: Send + Sync {
    /// Export all gRPC services this module wants to expose.
    ///
    /// The returned handles are opaque for the core; concrete modules and grpc_hub
    /// agree on the actual type stored in `inner`.
    async fn export_grpc_services(
        &self,
        ctx: &crate::context::ModuleCtx,
    ) -> anyhow::Result<Vec<GrpcServiceHandle>>;
}

/// Trait for the single gRPC hub module that hosts all gRPC services.
///
/// There should be exactly one module with the `grpc_hub` capability per process.
/// It is responsible for starting a single gRPC server and wiring all services into it.
#[async_trait]
pub trait GrpcHubModule: Send + Sync {
    /// Called once by the runtime after all modules are initialized.
    ///
    /// `services` contains all exported gRPC service handles from modules that have
    /// the `grpc` capability. The hub is responsible for binding a single port and
    /// wiring all services into one tonic::Server (or equivalent).
    async fn run_grpc_host(
        self: Arc<Self>,
        ctx: &crate::context::ModuleCtx,
        services: Vec<GrpcServiceHandle>,
    ) -> anyhow::Result<()>;
}
