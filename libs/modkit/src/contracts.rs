use async_trait::async_trait;
use axum::Router;
use tokio_util::sync::CancellationToken;

pub use crate::api::openapi_registry::OpenApiRegistry;

/// System module: receives runtime internals before init.
///
/// This trait is internal to modkit and only used by system modules
/// (those with the "system" capability). Normal user modules don't implement this.
pub trait SystemModule: Send + Sync {
    /// Wire system-level context into this module.
    ///
    /// Called once during runtime bootstrap, before init(), for all system modules.
    fn wire_system(&self, sys: &crate::runtime::SystemContext);
}

/// Core module: DI/wiring; do not rely on migrated schema here.
#[async_trait]
pub trait Module: Send + Sync + 'static {
    async fn init(&self, ctx: &crate::context::ModuleCtx) -> anyhow::Result<()>;
    fn as_any(&self) -> &dyn std::any::Any;

    /// Return self as a SystemModule if this module has the "system" capability.
    ///
    /// Default implementation returns None. System modules override this to return Some(self).
    fn as_system_module(&self) -> Option<&dyn SystemModule> {
        None
    }
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

/// Represents a gRPC service registration callback used by the gRPC hub.
///
/// Each module that exposes gRPC services provides one or more of these.
/// The `register` closure adds the service into the provided `RoutesBuilder`.
#[cfg(feature = "otel")]
pub struct RegisterGrpcServiceFn {
    pub service_name: &'static str,
    pub register: Box<dyn Fn(&mut tonic::service::RoutesBuilder) + Send + Sync>,
}

#[cfg(not(feature = "otel"))]
pub struct RegisterGrpcServiceFn {
    pub service_name: &'static str,
}

/// Trait for modules that export gRPC services.
///
/// The runtime will call this during the gRPC registration phase to collect
/// all services that should be exposed on the shared gRPC server.
#[async_trait]
pub trait GrpcServiceModule: Send + Sync {
    /// Returns all gRPC services this module wants to expose.
    ///
    /// Each installer adds one service to the tonic::Server builder.
    async fn get_grpc_services(
        &self,
        ctx: &crate::context::ModuleCtx,
    ) -> anyhow::Result<Vec<RegisterGrpcServiceFn>>;
}

/// Trait for the gRPC hub module that hosts the gRPC server.
///
/// This trait is implemented by the single module responsible for hosting
/// the tonic::Server instance. Only one module per process should implement this.
pub trait GrpcHubModule: Send + Sync {
    /// Returns the bound endpoint after the server starts listening.
    ///
    /// Examples:
    /// - TCP: `http://127.0.0.1:50652`
    /// - Unix socket: `unix:///path/to/socket`
    /// - Named pipe: `pipe://\\.\pipe\name`
    ///
    /// Returns `None` if the server hasn't started listening yet.
    fn bound_endpoint(&self) -> Option<String>;
}
