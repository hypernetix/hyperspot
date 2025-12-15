#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
//! gRPC Hub Module
//!
//! This module builds and hosts the single `tonic::Server` instance for the process.

use anyhow::Context;
use async_trait::async_trait;
use modkit::{
    context::ModuleCtx,
    contracts::{Module, SystemModule},
    lifecycle::ReadySignal,
    runtime::{GrpcInstallerData, GrpcInstallerStore, ModuleInstallers},
    DirectoryApi,
};

use parking_lot::RwLock;
use serde::Deserialize;
#[cfg(unix)]
use std::path::PathBuf;
use std::{
    collections::HashSet,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::Arc,
};
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tokio_util::sync::CancellationToken;
use tonic::{service::RoutesBuilder, transport::Server};

#[cfg(windows)]
use modkit_transport_grpc::create_named_pipe_incoming;

const DEFAULT_LISTEN_ADDR: SocketAddr =
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 50051));

/// Configuration for the gRPC Hub module.
///
/// Supports multiple transport types via `listen_addr`:
/// - TCP: `"127.0.0.1:50051"` or `"0.0.0.0:0"` for ephemeral port
/// - Unix Domain Socket (Unix only): `"uds:///path/to/socket.sock"`
/// - Named Pipe (Windows only): `"pipe://\\.\pipe\my_pipe"` or `"npipe://\\.\pipe\my_pipe"`
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct GrpcHubConfig {
    /// Listen address for the gRPC server.
    /// Defaults to `0.0.0.0:50051` if not specified.
    pub listen_addr: String,
}

impl Default for GrpcHubConfig {
    fn default() -> Self {
        Self {
            listen_addr: DEFAULT_LISTEN_ADDR.to_string(),
        }
    }
}

/// Configuration for the listen address
#[derive(Clone)]
enum ListenConfig {
    Tcp(SocketAddr),
    #[cfg(unix)]
    Uds(PathBuf),
    #[cfg(windows)]
    NamedPipe(String),
}

/// The gRPC Hub module.
/// This module is responsible for hosting the gRPC server and managing the gRPC services.
#[modkit::module(
    name = "grpc_hub",
    capabilities = [stateful, system, grpc_hub],
    lifecycle(entry = "serve", await_ready)
)]
pub struct GrpcHub {
    listen_cfg: RwLock<ListenConfig>,
    installer_store: RwLock<Option<Arc<GrpcInstallerStore>>>,
    directory: RwLock<Option<Arc<dyn DirectoryApi>>>,
    /// Process-level instance ID (set from `SystemContext` during `wire_system` phase)
    instance_id: RwLock<String>,
    bound_endpoint: RwLock<Option<String>>,
}

impl Default for GrpcHub {
    fn default() -> Self {
        Self {
            listen_cfg: RwLock::new(ListenConfig::Tcp(DEFAULT_LISTEN_ADDR)),
            installer_store: RwLock::new(None),
            directory: RwLock::new(None),
            instance_id: RwLock::new(String::new()), // Set from SystemContext during wire_system
            bound_endpoint: RwLock::new(None),
        }
    }
}

impl GrpcHub {
    /// Update the listen address to TCP (primarily used by tests/config).
    pub fn set_listen_addr_tcp(&self, addr: SocketAddr) {
        *self.listen_cfg.write() = ListenConfig::Tcp(addr);
    }

    /// Current TCP listen address (returns None if using UDS or named pipe).
    pub fn listen_addr_tcp(&self) -> Option<SocketAddr> {
        match *self.listen_cfg.read() {
            ListenConfig::Tcp(addr) => Some(addr),
            #[cfg(unix)]
            ListenConfig::Uds(_) => None,
            #[cfg(windows)]
            ListenConfig::NamedPipe(_) => None,
        }
    }

    /// Set listen address to Windows named pipe (primarily used by tests).
    #[cfg(windows)]
    pub fn set_listen_named_pipe(&self, name: impl Into<String>) {
        *self.listen_cfg.write() = ListenConfig::NamedPipe(name.into());
    }

    /// Get the actual bound endpoint after the server has started.
    ///
    /// Returns the full endpoint URL (e.g., `http://127.0.0.1:50652` for TCP,
    /// `unix:///path/to/socket` for UDS, or `pipe://\\.\pipe\name` for named pipes).
    /// Returns `None` if the server hasn't started yet.
    fn get_bound_endpoint(&self) -> Option<String> {
        self.bound_endpoint.read().clone()
    }

    /// Set the bound endpoint after the server has started listening.
    fn set_bound_endpoint(&self, endpoint: String) {
        *self.bound_endpoint.write() = Some(endpoint);
    }

    /// Parse and apply listen address configuration.
    ///
    /// Supports:
    /// - TCP: `"127.0.0.1:50051"` or `"0.0.0.0:0"` for ephemeral port
    /// - Unix Domain Socket (Unix only): `"uds:///path/to/socket.sock"`
    /// - Named Pipe (Windows only): `"pipe://\\.\pipe\my_pipe"` or `"npipe://\\.\pipe\my_pipe"`
    ///
    /// # Errors
    /// Returns an error if the address format is invalid or unsupported on the platform.
    pub fn apply_listen_config(&self, listen_addr: &str) -> anyhow::Result<()> {
        // First, try platform-specific parsing
        if self.apply_platform_specific(listen_addr)? {
            return Ok(());
        }

        // Fall back to TCP SocketAddr parsing
        let addr = listen_addr
            .parse::<SocketAddr>()
            .with_context(|| format!("invalid listen_addr '{listen_addr}'"))?;
        *self.listen_cfg.write() = ListenConfig::Tcp(addr);
        tracing::info!(%addr, "gRPC hub listen address configured for TCP");

        Ok(())
    }

    /// Platform-specific address parsing.
    ///
    /// Returns `Ok(true)` if the address was fully handled by this method,
    /// `Ok(false)` if the caller should fall back to TCP parsing.
    #[cfg(windows)]
    fn apply_platform_specific(&self, listen_addr: &str) -> anyhow::Result<bool> {
        // Handle Windows named pipes: pipe:// or npipe://
        if let Some(pipe_name) = listen_addr
            .strip_prefix("pipe://")
            .or_else(|| listen_addr.strip_prefix("npipe://"))
        {
            let pipe_name = pipe_name.to_owned();
            *self.listen_cfg.write() = ListenConfig::NamedPipe(pipe_name.clone());
            tracing::info!(
                name = %pipe_name,
                "gRPC hub listen address configured for Windows named pipe"
            );
            return Ok(true);
        }

        // Explicitly reject UDS on Windows
        if listen_addr.starts_with("uds://") {
            anyhow::bail!("UDS listen_addr is not supported on Windows: '{listen_addr}'");
        }

        // Not a platform-specific address, fall back to TCP
        Ok(false)
    }

    /// Platform-specific address parsing.
    ///
    /// Returns `Ok(true)` if the address was fully handled by this method,
    /// `Ok(false)` if the caller should fall back to TCP parsing.
    #[cfg(unix)]
    fn apply_platform_specific(&self, listen_addr: &str) -> anyhow::Result<bool> {
        // Explicitly reject named pipes on Unix
        if listen_addr.starts_with("pipe://") || listen_addr.starts_with("npipe://") {
            tracing::warn!(
                listen_addr = %listen_addr,
                "Named pipe listen_addr is configured but named pipes are not supported on this platform"
            );
            anyhow::bail!(
                "Named pipe listen_addr is not supported on this platform: '{listen_addr}'"
            );
        }

        // Handle Unix Domain Sockets: uds://
        if let Some(uds_path) = listen_addr.strip_prefix("uds://") {
            let path = std::path::PathBuf::from(uds_path);
            *self.listen_cfg.write() = ListenConfig::Uds(path.clone());
            tracing::info!(
                path = %path.display(),
                "gRPC hub listen address configured for UDS"
            );
            return Ok(true);
        }

        // Not a platform-specific address, fall back to TCP
        Ok(false)
    }

    /// Validate that all service names are unique across all modules.
    fn validate_unique_services(modules: &[ModuleInstallers]) -> anyhow::Result<()> {
        let mut seen = HashSet::new();
        for module in modules {
            for installer in &module.installers {
                if !seen.insert(installer.service_name) {
                    anyhow::bail!(
                        "Duplicate gRPC service detected: {}",
                        installer.service_name
                    );
                }
            }
        }
        Ok(())
    }

    /// Build routes from module installers. Returns None if no services registered.
    fn build_routes_from_modules(modules: &[ModuleInstallers]) -> Option<tonic::service::Routes> {
        let mut routes_builder = RoutesBuilder::default();
        let mut has_services = false;
        for module in modules {
            for installer in &module.installers {
                (installer.register)(&mut routes_builder);
                has_services = true;
            }
        }
        if has_services {
            Some(routes_builder.routes())
        } else {
            None
        }
    }

    /// Prepare Unix Domain Socket path by removing existing socket file if present.
    #[cfg(unix)]
    fn prepare_uds_socket_path(path: &std::path::Path) {
        use std::io;

        if !path.exists() {
            return;
        }

        match std::fs::remove_file(path) {
            Ok(()) => {
                tracing::debug!(
                    path = %path.display(),
                    "removed existing UDS socket file before bind"
                );
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => {
                tracing::warn!(
                    path = %path.display(),
                    error = %e,
                    "failed to remove existing UDS socket file before bind"
                );
            }
        }
    }

    /// Deregister modules from Directory on shutdown.
    async fn deregister_modules(&self, modules: &[ModuleInstallers]) {
        let directory = {
            let guard = self.directory.read();
            guard.as_ref().cloned()
        };
        let instance_id = self.instance_id.read().clone();

        if let Some(directory) = directory {
            for module_data in modules {
                if let Err(e) = directory
                    .deregister_instance(&module_data.module_name, &instance_id)
                    .await
                {
                    tracing::warn!(
                        module = %module_data.module_name,
                        error = %e,
                        "Failed to deregister module from Directory"
                    );
                }
            }
        }
    }

    /// Run the tonic server with the provided installers.
    ///
    /// # Errors
    /// Returns an error if server startup or execution fails.
    pub async fn run_with_installers(
        &self,
        data: GrpcInstallerData,
        cancel: CancellationToken,
        ready: ReadySignal,
    ) -> anyhow::Result<()> {
        Self::validate_unique_services(&data.modules)?;

        let Some(routes) = Self::build_routes_from_modules(&data.modules) else {
            ready.notify();
            cancel.cancelled().await;
            return Ok(());
        };

        let listen_cfg = self.listen_cfg.read().clone();
        let serve_result = match listen_cfg {
            ListenConfig::Tcp(addr) => {
                self.serve_tcp(addr, routes, &data.modules, cancel, ready)
                    .await
            }
            #[cfg(unix)]
            ListenConfig::Uds(path) => {
                self.serve_uds(path, routes, &data.modules, cancel, ready)
                    .await
            }
            #[cfg(windows)]
            ListenConfig::NamedPipe(ref pipe_name) => {
                self.serve_named_pipe(pipe_name.clone(), routes, &data.modules, cancel, ready)
                    .await
            }
        };

        self.deregister_modules(&data.modules).await;
        serve_result
    }

    /// Serve gRPC over TCP with Directory registration.
    async fn serve_tcp(
        &self,
        addr: SocketAddr,
        routes: tonic::service::Routes,
        modules: &[ModuleInstallers],
        cancel: CancellationToken,
        ready: ReadySignal,
    ) -> anyhow::Result<()> {
        let listener = TcpListener::bind(addr).await?;
        let bound_addr = listener.local_addr()?;
        let endpoint = format!("http://{bound_addr}");
        tracing::info!(%bound_addr, transport = "tcp", "gRPC hub listening");

        self.set_bound_endpoint(endpoint.clone());
        self.register_modules(modules, &endpoint).await?;
        ready.notify();

        let incoming = TcpListenerStream::new(listener);
        Server::builder()
            .add_routes(routes)
            .serve_with_incoming_shutdown(incoming, async move {
                cancel.cancelled().await;
            })
            .await?;
        Ok(())
    }

    /// Serve gRPC over Unix Domain Socket with Directory registration.
    #[cfg(unix)]
    async fn serve_uds(
        &self,
        path: std::path::PathBuf,
        routes: tonic::service::Routes,
        modules: &[ModuleInstallers],
        cancel: CancellationToken,
        ready: ReadySignal,
    ) -> anyhow::Result<()> {
        use tokio::net::UnixListener;
        use tokio_stream::wrappers::UnixListenerStream;

        Self::prepare_uds_socket_path(&path);

        tracing::info!(
            path = %path.display(),
            transport = "uds",
            "gRPC hub listening"
        );

        let uds = UnixListener::bind(&path)
            .with_context(|| format!("failed to bind UDS listener at '{}'", path.display()))?;

        let endpoint = format!("unix://{}", path.display());
        self.set_bound_endpoint(endpoint.clone());
        self.register_modules(modules, &endpoint).await?;
        ready.notify();

        let incoming = UnixListenerStream::new(uds);
        Server::builder()
            .add_routes(routes)
            .serve_with_incoming_shutdown(incoming, async move {
                cancel.cancelled().await;
            })
            .await?;
        Ok(())
    }

    /// Serve gRPC over Windows named pipe with Directory registration.
    #[cfg(windows)]
    async fn serve_named_pipe(
        &self,
        pipe_name: String,
        routes: tonic::service::Routes,
        modules: &[ModuleInstallers],
        cancel: CancellationToken,
        ready: ReadySignal,
    ) -> anyhow::Result<()> {
        tracing::info!(name = %pipe_name, transport = "named_pipe", "gRPC hub listening");

        let endpoint = format!("pipe://{pipe_name}");
        self.set_bound_endpoint(endpoint.clone());
        self.register_modules(modules, &endpoint).await?;
        ready.notify();

        let incoming = create_named_pipe_incoming(pipe_name, cancel.clone());
        Server::builder()
            .add_routes(routes)
            .serve_with_incoming_shutdown(incoming, async move {
                cancel.cancelled().await;
            })
            .await?;
        Ok(())
    }

    async fn register_modules(
        &self,
        modules: &[ModuleInstallers],
        endpoint: &str,
    ) -> anyhow::Result<()> {
        let directory = {
            let guard = self.directory.read();
            guard.as_ref().cloned()
        };
        let instance_id = self.instance_id.read().clone();

        if let Some(directory) = directory {
            for module_data in modules {
                let service_names: Vec<String> = module_data
                    .installers
                    .iter()
                    .map(|i| i.service_name.to_owned())
                    .collect();

                let info = module_orchestrator_contracts::RegisterInstanceInfo {
                    module: module_data.module_name.clone(),
                    instance_id: instance_id.clone(),
                    grpc_services: service_names
                        .iter()
                        .map(|n| {
                            (
                                n.clone(),
                                module_orchestrator_contracts::ServiceEndpoint::new(endpoint),
                            )
                        })
                        .collect(),
                    version: Some(env!("CARGO_PKG_VERSION").to_owned()),
                };

                directory.register_instance(info).await?;
                tracing::info!(
                    module = %module_data.module_name,
                    endpoint = %endpoint,
                    "Registered module in Directory"
                );
            }
        } else {
            tracing::info!("DirectoryApi not available; skipping Directory registration");
        }

        Ok(())
    }

    async fn serve(
        self: Arc<Self>,
        cancel: CancellationToken,
        ready: ReadySignal,
    ) -> anyhow::Result<()> {
        let data = {
            let store_guard = self.installer_store.read();
            let store = store_guard
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("GrpcInstallerStore not wired into GrpcHub"))?;
            store.take()
        };

        let data = data.ok_or_else(|| anyhow::anyhow!("GrpcInstallerStore is empty"))?;

        self.run_with_installers(data, cancel, ready).await
    }
}

impl SystemModule for GrpcHub {
    fn wire_system(&self, sys: &modkit::runtime::SystemContext) {
        // Store gRPC installer store reference
        {
            let mut guard = self.installer_store.write();
            *guard = Some(Arc::clone(&sys.grpc_installers));
        }
        // Store process-level instance_id from SystemContext
        {
            let mut guard = self.instance_id.write();
            *guard = sys.instance_id().to_string();
        }
    }
}

impl modkit::contracts::GrpcHubModule for GrpcHub {
    fn bound_endpoint(&self) -> Option<String> {
        self.get_bound_endpoint()
    }
}

#[async_trait]
impl Module for GrpcHub {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        // Load typed configuration
        let cfg: GrpcHubConfig = ctx.config()?;
        tracing::debug!(listen_addr = %cfg.listen_addr, "Loaded gRPC hub configuration");

        // Parse listen_addr into appropriate transport type
        self.apply_listen_config(&cfg.listen_addr)?;

        // Fetch DirectoryApi from ClientHub if available
        if let Ok(dir) = ctx.client_hub().get::<dyn DirectoryApi>() {
            *self.directory.write() = Some(dir);
        }

        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_system_module(&self) -> Option<&dyn SystemModule> {
        Some(self)
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use http::{Request, Response};
    use modkit::runtime::ModuleInstallers;
    use modkit::{client_hub::ClientHub, config::ConfigProvider, context::ModuleCtx};
    use std::{
        convert::Infallible,
        future,
        sync::Arc,
        task::{Context as TaskContext, Poll},
    };
    use tokio::time::{sleep, Duration};
    use tonic::{body::Body, server::NamedService};
    use tower::Service;
    use uuid::Uuid;

    const SERVICE_A: &str = "grpc_hub.test.ServiceA";
    const SERVICE_B: &str = "grpc_hub.test.ServiceB";

    #[derive(Clone)]
    struct ServiceAImpl;

    #[derive(Clone)]
    struct ServiceBImpl;

    impl NamedService for ServiceAImpl {
        const NAME: &'static str = SERVICE_A;
    }

    impl NamedService for ServiceBImpl {
        const NAME: &'static str = SERVICE_B;
    }

    impl Service<Request<Body>> for ServiceAImpl {
        type Response = Response<Body>;
        type Error = Infallible;
        type Future = future::Ready<Result<Self::Response, Self::Error>>;

        fn poll_ready(&mut self, _cx: &mut TaskContext<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, _req: Request<Body>) -> Self::Future {
            future::ready(Ok(Response::new(Body::empty())))
        }
    }

    impl Service<Request<Body>> for ServiceBImpl {
        type Response = Response<Body>;
        type Error = Infallible;
        type Future = future::Ready<Result<Self::Response, Self::Error>>;

        fn poll_ready(&mut self, _cx: &mut TaskContext<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, _req: Request<Body>) -> Self::Future {
            future::ready(Ok(Response::new(Body::empty())))
        }
    }

    fn installer_a() -> modkit::contracts::RegisterGrpcServiceFn {
        modkit::contracts::RegisterGrpcServiceFn {
            service_name: SERVICE_A,
            register: Box::new(|routes| {
                routes.add_service(ServiceAImpl);
            }),
        }
    }

    fn installer_b() -> modkit::contracts::RegisterGrpcServiceFn {
        modkit::contracts::RegisterGrpcServiceFn {
            service_name: SERVICE_B,
            register: Box::new(|routes| {
                routes.add_service(ServiceBImpl);
            }),
        }
    }

    #[tokio::test]
    async fn test_run_with_installers_rejects_duplicates() {
        let hub = GrpcHub::default();
        hub.set_listen_addr_tcp("127.0.0.1:0".parse().unwrap());
        let data = GrpcInstallerData {
            modules: vec![ModuleInstallers {
                module_name: "test".to_owned(),
                installers: vec![installer_a(), installer_a()],
            }],
        };
        let cancel = CancellationToken::new();
        let (tx, _rx) = tokio::sync::oneshot::channel();
        let ready = ReadySignal::from_sender(tx);

        let result = hub.run_with_installers(data, cancel, ready).await;

        assert!(result.is_err(), "duplicate services should error");
    }

    #[tokio::test]
    async fn test_run_with_installers_starts_server() {
        let hub = Arc::new(GrpcHub::default());
        hub.set_listen_addr_tcp("127.0.0.1:0".parse().unwrap());
        let data = GrpcInstallerData {
            modules: vec![ModuleInstallers {
                module_name: "test".to_owned(),
                installers: vec![installer_a(), installer_b()],
            }],
        };
        let cancel = CancellationToken::new();
        let cancel_clone = cancel.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();
        let ready = ReadySignal::from_sender(tx);

        let hub_task = {
            let hub = hub.clone();
            tokio::spawn(async move { hub.run_with_installers(data, cancel, ready).await })
        };

        tokio::spawn(async move {
            sleep(Duration::from_millis(50)).await;
            cancel_clone.cancel();
        });

        tokio::time::timeout(Duration::from_secs(1), rx)
            .await
            .expect("ready signal should fire")
            .expect("ready channel should complete");

        hub_task
            .await
            .expect("task should join successfully")
            .expect("server should exit cleanly");
    }

    #[tokio::test]
    async fn test_serve_with_system_context() {
        let hub = Arc::new(GrpcHub::default());
        hub.set_listen_addr_tcp("127.0.0.1:0".parse().unwrap());

        // Wire system context with installers
        let installer_store = Arc::new(GrpcInstallerStore::new());
        installer_store
            .set(GrpcInstallerData {
                modules: vec![ModuleInstallers {
                    module_name: "test".to_owned(),
                    installers: vec![installer_a()],
                }],
            })
            .expect("store should accept installers");

        let module_manager = Arc::new(modkit::runtime::ModuleManager::new());
        let sys_ctx = modkit::runtime::SystemContext::new(
            Uuid::new_v4(),
            module_manager,
            Arc::clone(&installer_store),
        );

        hub.wire_system(&sys_ctx);

        let cancel = CancellationToken::new();
        let cancel_clone = cancel.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();
        let ready = ReadySignal::from_sender(tx);

        let serve_task = {
            let hub = hub.clone();
            tokio::spawn(async move { hub.serve(cancel, ready).await })
        };

        tokio::spawn(async move {
            sleep(Duration::from_millis(50)).await;
            cancel_clone.cancel();
        });

        tokio::time::timeout(Duration::from_secs(1), rx)
            .await
            .expect("ready signal should fire")
            .expect("ready signal should complete");

        serve_task
            .await
            .expect("task should join")
            .expect("serve should complete without error");

        // After serve completes, installer_store should be empty (consumed)
        assert!(
            installer_store.is_empty(),
            "installers should be consumed after serve completes"
        );
    }

    #[tokio::test]
    async fn test_init_parses_listen_addr() {
        #[derive(Default)]
        struct ConfigProviderWithAddr;
        impl ConfigProvider for ConfigProviderWithAddr {
            fn get_module_config(&self, module_name: &str) -> Option<&serde_json::Value> {
                if module_name == "grpc_hub" {
                    use std::sync::OnceLock;
                    static CONFIG: OnceLock<serde_json::Value> = OnceLock::new();
                    Some(CONFIG.get_or_init(|| {
                        serde_json::json!({
                            "config": {
                                "listen_addr": "127.0.0.1:10"
                            }
                        })
                    }))
                } else {
                    None
                }
            }
        }

        let hub = GrpcHub::default();
        let cancel = CancellationToken::new();

        let ctx = ModuleCtx::new(
            "grpc_hub",
            Uuid::new_v4(),
            Arc::new(ConfigProviderWithAddr),
            Arc::new(ClientHub::default()),
            cancel,
            None,
        );

        hub.init(&ctx).await.expect("init should succeed");

        assert_eq!(
            hub.listen_addr_tcp().expect("should be TCP"),
            "127.0.0.1:10".parse().unwrap()
        );
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn test_init_parses_uds_addr() {
        #[derive(Default)]
        struct ConfigProviderWithUds;
        impl ConfigProvider for ConfigProviderWithUds {
            fn get_module_config(&self, module_name: &str) -> Option<&serde_json::Value> {
                if module_name == "grpc_hub" {
                    use std::sync::OnceLock;
                    static CONFIG: OnceLock<serde_json::Value> = OnceLock::new();
                    Some(CONFIG.get_or_init(|| {
                        serde_json::json!({
                            "config": {
                                "listen_addr": "uds:///tmp/test_grpc.sock"
                            }
                        })
                    }))
                } else {
                    None
                }
            }
        }

        let hub = GrpcHub::default();
        let cancel = CancellationToken::new();

        let ctx = ModuleCtx::new(
            "grpc_hub",
            Uuid::new_v4(),
            Arc::new(ConfigProviderWithUds),
            Arc::new(ClientHub::default()),
            cancel,
            None,
        );

        hub.init(&ctx).await.expect("init should succeed");

        // Verify that listen_addr_tcp returns None for UDS config
        assert!(
            hub.listen_addr_tcp().is_none(),
            "Expected UDS config, not TCP"
        );
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn test_init_parses_uds_listen_addr_and_serves() {
        use tempfile::TempDir;

        // Custom ConfigProvider returning uds:// path
        struct ConfigProviderWithUds {
            config_value: serde_json::Value,
        }
        impl ConfigProvider for ConfigProviderWithUds {
            fn get_module_config(&self, module_name: &str) -> Option<&serde_json::Value> {
                if module_name == "grpc_hub" {
                    Some(&self.config_value)
                } else {
                    None
                }
            }
        }

        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let socket_path = temp_dir.path().join("test_grpc_hub.sock");
        let socket_path_str = format!("uds://{}", socket_path.display());

        let hub = Arc::new(GrpcHub::default());
        let cancel = CancellationToken::new();

        let config_provider = ConfigProviderWithUds {
            config_value: serde_json::json!({
                "config": {
                    "listen_addr": socket_path_str
                }
            }),
        };

        let ctx = ModuleCtx::new(
            "grpc_hub",
            Uuid::new_v4(),
            Arc::new(config_provider),
            Arc::new(ClientHub::default()),
            cancel.clone(),
            None,
        );

        hub.init(&ctx).await.expect("init should succeed");

        let installers = vec![installer_a()];
        let data = GrpcInstallerData {
            modules: vec![ModuleInstallers {
                module_name: "test".to_owned(),
                installers,
            }],
        };
        let cancel_clone = cancel.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();
        let ready = ReadySignal::from_sender(tx);

        let hub_task = {
            let hub = hub.clone();
            tokio::spawn(async move { hub.run_with_installers(data, cancel, ready).await })
        };

        tokio::spawn(async move {
            sleep(Duration::from_millis(100)).await;
            cancel_clone.cancel();
        });

        tokio::time::timeout(Duration::from_secs(2), rx)
            .await
            .expect("ready signal should fire")
            .expect("ready channel should complete");

        // Verify socket file was created
        assert!(socket_path.exists(), "Unix socket file should be created");

        hub_task
            .await
            .expect("task should join successfully")
            .expect("server should exit cleanly");
    }

    #[tokio::test]
    #[cfg(windows)]
    async fn test_named_pipe_listen_and_shutdown() {
        // Custom ConfigProvider returning named pipe address
        struct ConfigProviderWithNamedPipe;
        impl ConfigProvider for ConfigProviderWithNamedPipe {
            fn get_module_config(&self, module_name: &str) -> Option<&serde_json::Value> {
                if module_name == "grpc_hub" {
                    use std::sync::OnceLock;
                    static CONFIG: OnceLock<serde_json::Value> = OnceLock::new();
                    Some(CONFIG.get_or_init(|| {
                        serde_json::json!({
                            "config": {
                                "listen_addr": r"pipe://\\.\pipe\test_grpc_hub"
                            }
                        })
                    }))
                } else {
                    None
                }
            }
        }

        let hub = Arc::new(GrpcHub::default());
        let cancel = CancellationToken::new();

        let ctx = ModuleCtx::new(
            "grpc_hub",
            Uuid::new_v4(),
            Arc::new(ConfigProviderWithNamedPipe),
            Arc::new(ClientHub::default()),
            cancel.clone(),
            None,
        );

        hub.init(&ctx).await.expect("init should succeed");

        // Verify that listen_addr_tcp returns None for named pipe config
        assert!(
            hub.listen_addr_tcp().is_none(),
            "Expected named pipe config, not TCP"
        );

        let installers = vec![installer_a()];
        let data = GrpcInstallerData {
            modules: vec![ModuleInstallers {
                module_name: "test".to_owned(),
                installers,
            }],
        };
        let cancel_clone = cancel.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();
        let ready = ReadySignal::from_sender(tx);

        let hub_task = {
            let hub = hub.clone();
            tokio::spawn(async move { hub.run_with_installers(data, cancel, ready).await })
        };

        // Give the server a moment to start, then cancel
        tokio::spawn(async move {
            sleep(Duration::from_millis(100)).await;
            cancel_clone.cancel();
        });

        tokio::time::timeout(Duration::from_secs(2), rx)
            .await
            .expect("ready signal should fire")
            .expect("ready channel should complete");

        hub_task
            .await
            .expect("task should join successfully")
            .expect("server should exit cleanly");
    }

    #[tokio::test]
    async fn test_run_with_no_installers_exits_gracefully() {
        let hub = GrpcHub::default();
        hub.set_listen_addr_tcp("127.0.0.1:0".parse().unwrap());
        let data = GrpcInstallerData { modules: vec![] };
        let cancel = CancellationToken::new();
        let cancel_clone = cancel.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();
        let ready = ReadySignal::from_sender(tx);

        let hub_task =
            tokio::spawn(async move { hub.run_with_installers(data, cancel, ready).await });

        // Schedule cancellation
        tokio::spawn(async move {
            sleep(Duration::from_millis(50)).await;
            cancel_clone.cancel();
        });

        // Should receive ready signal immediately
        tokio::time::timeout(Duration::from_secs(1), rx)
            .await
            .expect("ready signal should fire")
            .expect("ready channel should complete");

        // Task should complete successfully
        hub_task
            .await
            .expect("task should join successfully")
            .expect("should exit cleanly with no services");
    }
}
