//! gRPC Hub Module
//!
//! This module builds and hosts the single tonic::Server instance for the process.

use anyhow::Context;
use async_trait::async_trait;
use modkit::{
    context::ModuleCtx,
    contracts::{Module, RegisterGrpcServiceFn, SystemModule},
    lifecycle::ReadySignal,
    runtime::GrpcInstallerStore,
};
use parking_lot::RwLock;
#[cfg(unix)]
use std::path::PathBuf;
use std::{
    collections::HashSet,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::Arc,
};
use tokio_util::sync::CancellationToken;
use tonic::{service::Routes, service::RoutesBuilder, transport::Server};

#[cfg(windows)]
use modkit_transport_grpc::create_named_pipe_incoming;

const DEFAULT_LISTEN_ADDR: SocketAddr =
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 50051));

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
/// Declared with capabilities: [stateful, system, grpc_hub].
#[modkit::module(
    name = "grpc_hub",
    capabilities = [stateful, system, grpc_hub],
    lifecycle(entry = "serve", await_ready)
)]
pub struct GrpcHub {
    listen_cfg: RwLock<ListenConfig>,
    installer_store: RwLock<Option<Arc<GrpcInstallerStore>>>,
}

impl Default for GrpcHub {
    fn default() -> Self {
        Self {
            listen_cfg: RwLock::new(ListenConfig::Tcp(DEFAULT_LISTEN_ADDR)),
            installer_store: RwLock::new(None),
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

    /// Validate that all service names are unique.
    fn validate_unique_services(installers: &[RegisterGrpcServiceFn]) -> anyhow::Result<()> {
        let mut seen = HashSet::new();
        for installer in installers {
            if !seen.insert(installer.service_name) {
                anyhow::bail!(
                    "Duplicate gRPC service detected: {}",
                    installer.service_name
                );
            }
        }
        Ok(())
    }

    /// Build routes from installers. Returns None if no services were registered.
    fn build_routes(installers: Vec<RegisterGrpcServiceFn>) -> Option<Routes> {
        if installers.is_empty() {
            return None;
        }

        let mut routes_builder = RoutesBuilder::default();
        for installer in installers {
            (installer.register)(&mut routes_builder);
        }
        Some(routes_builder.routes())
    }

    /// Prepare Unix Domain Socket path by removing existing socket file if present.
    ///
    /// This handles the cleanup of stale socket files that may exist from previous
    /// server instances. Logs appropriate messages for different scenarios.
    #[cfg(unix)]
    fn prepare_uds_socket_path(path: &std::path::Path) -> std::io::Result<()> {
        use std::io;

        if !path.exists() {
            return Ok(());
        }

        match std::fs::remove_file(path) {
            Ok(_) => {
                tracing::debug!(
                    path = %path.display(),
                    "removed existing UDS socket file before bind"
                );
                Ok(())
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                // Lost the race, somebody else removed it. Not fatal.
                Ok(())
            }
            Err(e) => {
                tracing::warn!(
                    path = %path.display(),
                    error = %e,
                    "failed to remove existing UDS socket file before bind"
                );
                Err(e)
            }
        }
    }

    /// Serve gRPC over TCP.
    async fn serve_tcp(
        addr: SocketAddr,
        routes: Routes,
        cancel: CancellationToken,
    ) -> anyhow::Result<()> {
        tracing::info!(%addr, transport = "tcp", "gRPC hub listening");
        Server::builder()
            .add_routes(routes)
            .serve_with_shutdown(addr, async move {
                cancel.cancelled().await;
            })
            .await?;
        Ok(())
    }

    /// Serve gRPC over Unix Domain Socket.
    #[cfg(unix)]
    async fn serve_uds(
        path: PathBuf,
        routes: Routes,
        cancel: CancellationToken,
    ) -> anyhow::Result<()> {
        use tokio::net::UnixListener;
        use tokio_stream::wrappers::UnixListenerStream;

        // Cleanup any existing socket file
        Self::prepare_uds_socket_path(&path)?;

        tracing::info!(
            path = %path.display(),
            transport = "uds",
            "gRPC hub listening"
        );

        let uds = UnixListener::bind(&path)
            .with_context(|| format!("failed to bind UDS listener at '{}'", path.display()))?;

        let incoming = UnixListenerStream::new(uds);

        Server::builder()
            .add_routes(routes)
            .serve_with_incoming_shutdown(incoming, async move {
                cancel.cancelled().await;
            })
            .await?;
        Ok(())
    }

    /// Serve gRPC over Windows named pipe.
    #[cfg(windows)]
    async fn serve_named_pipe(
        pipe_name: String,
        routes: Routes,
        cancel: CancellationToken,
    ) -> anyhow::Result<()> {
        tracing::info!(name = %pipe_name, transport = "named_pipe", "gRPC hub listening");

        let incoming = create_named_pipe_incoming(pipe_name, cancel.clone());

        Server::builder()
            .add_routes(routes)
            .serve_with_incoming_shutdown(incoming, async move {
                cancel.cancelled().await;
            })
            .await?;
        Ok(())
    }

    /// Run the tonic server with the provided installers.
    pub async fn run_with_installers(
        &self,
        installers: Vec<RegisterGrpcServiceFn>,
        cancel: CancellationToken,
        ready: ReadySignal,
    ) -> anyhow::Result<()> {
        Self::validate_unique_services(&installers)?;

        let routes = match Self::build_routes(installers) {
            Some(r) => r,
            None => {
                // No services to serve, notify ready and wait for cancellation
                ready.notify();
                cancel.cancelled().await;
                return Ok(());
            }
        };

        // NOTE: With tonic 0.14.2's API, serve_with_shutdown binds internally.
        // We notify ready after building routes but before serve starts.
        // This is better than notifying before route building, but not as strict as binding first.
        // For stricter control, consider using a lower-level API or upgrading tonic.
        ready.notify();

        let listen_cfg = self.listen_cfg.read().clone();

        match listen_cfg {
            ListenConfig::Tcp(addr) => Self::serve_tcp(addr, routes, cancel).await?,
            #[cfg(unix)]
            ListenConfig::Uds(path) => Self::serve_uds(path, routes, cancel).await?,
            #[cfg(windows)]
            ListenConfig::NamedPipe(pipe_name) => {
                Self::serve_named_pipe(pipe_name, routes, cancel).await?;
            }
        }

        Ok(())
    }

    async fn serve(
        self: Arc<Self>,
        cancel: CancellationToken,
        ready: ReadySignal,
    ) -> anyhow::Result<()> {
        let installers = {
            let store_guard = self.installer_store.read();
            let store = store_guard
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("GrpcInstallerStore not wired into GrpcHub"))?;
            store.take()
        };

        self.run_with_installers(installers, cancel, ready).await
    }
}

impl SystemModule for GrpcHub {
    fn wire_system(&self, sys: &modkit::runtime::SystemContext) {
        let mut guard = self.installer_store.write();
        *guard = Some(Arc::clone(&sys.grpc_installers));
    }
}

impl modkit::contracts::GrpcHubModule for GrpcHub {}

#[async_trait]
impl Module for GrpcHub {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        if let Some(addr_value) = ctx.raw_config().get("listen_addr") {
            if let Some(s) = addr_value.as_str() {
                // 1) Windows named pipes: pipe:// or npipe://
                #[cfg(windows)]
                if let Some(pipe_name) = s
                    .strip_prefix("pipe://")
                    .or_else(|| s.strip_prefix("npipe://"))
                {
                    let pipe_name = pipe_name.to_string();
                    *self.listen_cfg.write() = ListenConfig::NamedPipe(pipe_name.clone());
                    tracing::info!(
                        name = %pipe_name,
                        "gRPC hub listen address configured for Windows named pipe"
                    );
                    return Ok(());
                }

                // Non-Windows branch
                #[cfg(not(windows))]
                if s.starts_with("pipe://") || s.starts_with("npipe://") {
                    tracing::warn!(
                        listen_addr = %s,
                        "Named pipe listen_addr is configured but named pipes are not supported on this platform; keeping previous listen address"
                    );
                    return Ok(());
                }

                // 2) Unix UDS: uds://
                if let Some(_uds_path) = s.strip_prefix("uds://") {
                    #[cfg(unix)]
                    {
                        let path = std::path::PathBuf::from(_uds_path);
                        *self.listen_cfg.write() = ListenConfig::Uds(path.clone());
                        tracing::info!(
                            path = %path.display(),
                            "gRPC hub listen address configured for UDS"
                        );
                        return Ok(());
                    }
                    #[cfg(not(unix))]
                    {
                        anyhow::bail!("UDS listen_addr is not supported on this platform: '{}'", s);
                    }
                }

                // 3) Default: TCP SocketAddr
                let addr = s
                    .parse::<SocketAddr>()
                    .with_context(|| format!("invalid listen_addr '{s}'"))?;
                *self.listen_cfg.write() = ListenConfig::Tcp(addr);
                tracing::info!(%addr, "gRPC hub listen address configured for TCP");
            } else {
                tracing::warn!(
                    "gRPC hub config 'listen_addr' found but was not a valid string; using previous listen address"
                );
            }
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
mod tests {
    use super::*;
    use http::{Request, Response};
    use modkit::{
        client_hub::ClientHub, config::ConfigProvider, context::ModuleCtx, registry::ModuleRegistry,
    };
    use std::{
        convert::Infallible,
        future,
        sync::Arc,
        task::{Context as TaskContext, Poll},
    };
    use tokio::time::{sleep, Duration};
    use tonic::{body::Body, server::NamedService};
    use tower::Service;

    #[tokio::test]
    async fn test_grpc_hub_is_discoverable() {
        let registry = ModuleRegistry::discover_and_build().expect("registry build failed");

        let has_grpc_hub = registry
            .modules()
            .iter()
            .any(|e| e.name == "grpc_hub" && e.is_system && e.is_grpc_hub);

        assert!(
            has_grpc_hub,
            "grpc_hub module should be registered with system and grpc_hub capabilities"
        );
    }

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

    fn installer_a() -> RegisterGrpcServiceFn {
        RegisterGrpcServiceFn {
            service_name: SERVICE_A,
            register: Box::new(|routes| {
                routes.add_service(ServiceAImpl);
            }),
        }
    }

    fn installer_b() -> RegisterGrpcServiceFn {
        RegisterGrpcServiceFn {
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
        let installers = vec![installer_a(), installer_a()];
        let cancel = CancellationToken::new();
        let (tx, _rx) = tokio::sync::oneshot::channel();
        let ready = ReadySignal::from_sender(tx);

        let result = hub.run_with_installers(installers, cancel, ready).await;

        assert!(result.is_err(), "duplicate services should error");
    }

    #[tokio::test]
    async fn test_run_with_installers_starts_server() {
        let hub = Arc::new(GrpcHub::default());
        hub.set_listen_addr_tcp("127.0.0.1:0".parse().unwrap());
        let installers = vec![installer_a(), installer_b()];
        let cancel = CancellationToken::new();
        let cancel_clone = cancel.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();
        let ready = ReadySignal::from_sender(tx);

        let hub_task = {
            let hub = hub.clone();
            tokio::spawn(async move { hub.run_with_installers(installers, cancel, ready).await })
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
            .set(vec![installer_a()])
            .expect("store should accept installers");

        let module_manager = Arc::new(modkit::runtime::ModuleManager::new());
        let sys_ctx =
            modkit::runtime::SystemContext::new(module_manager, Arc::clone(&installer_store));

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
        let hub = GrpcHub::default();
        let cancel = CancellationToken::new();

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

        let ctx = ModuleCtx::new(
            "grpc_hub",
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
        let hub = GrpcHub::default();
        let cancel = CancellationToken::new();

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

        let ctx = ModuleCtx::new(
            "grpc_hub",
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

        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let socket_path = temp_dir.path().join("test_grpc_hub.sock");
        let socket_path_str = format!("uds://{}", socket_path.display());

        let hub = Arc::new(GrpcHub::default());
        let cancel = CancellationToken::new();

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

        let config_provider = ConfigProviderWithUds {
            config_value: serde_json::json!({
                "config": {
                    "listen_addr": socket_path_str
                }
            }),
        };

        let ctx = ModuleCtx::new(
            "grpc_hub",
            Arc::new(config_provider),
            Arc::new(ClientHub::default()),
            cancel.clone(),
            None,
        );

        hub.init(&ctx).await.expect("init should succeed");

        let installers = vec![installer_a()];
        let cancel_clone = cancel.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();
        let ready = ReadySignal::from_sender(tx);

        let hub_task = {
            let hub = hub.clone();
            tokio::spawn(async move { hub.run_with_installers(installers, cancel, ready).await })
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
        let hub = Arc::new(GrpcHub::default());
        let cancel = CancellationToken::new();

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

        let ctx = ModuleCtx::new(
            "grpc_hub",
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
        let cancel_clone = cancel.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();
        let ready = ReadySignal::from_sender(tx);

        let hub_task = {
            let hub = hub.clone();
            tokio::spawn(async move { hub.run_with_installers(installers, cancel, ready).await })
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
        let installers = vec![];
        let cancel = CancellationToken::new();
        let cancel_clone = cancel.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();
        let ready = ReadySignal::from_sender(tx);

        let hub_task =
            tokio::spawn(async move { hub.run_with_installers(installers, cancel, ready).await });

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
