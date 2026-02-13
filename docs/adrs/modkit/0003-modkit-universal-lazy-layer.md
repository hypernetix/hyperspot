# Proposal: Universal Lazy Typed Clients for OoP gRPC Modules

## Executive Summary

This proposal details the implementation of **universal lazy typed clients** for out-of-process (OoP) gRPC modules in ModKit. The solution eliminates fragile eager wiring during module startup, enabling graceful degradation when OoP dependencies are unavailable.

**Chosen approach**: Universal lazy layer in ModKit + code generation macro + `clients` declarations in `#[modkit::module]`.

## Problem Statement

The current OoP client wiring pattern has several issues:

1. **Eager wiring is fragile**: Consumer modules call `wire_client()` in `init()`, which fails if the OoP dependency is not yet available.
2. **Startup coupling**: The entire module fails to start if any OoP dependency is temporarily unavailable.
3. **Boilerplate duplication**: Each SDK repeats the same resolve/connect/cache logic.
4. **No graceful degradation**: Missing dependencies cause module-level failures instead of per-operation failures (HTTP 424).

### Current Pattern (calculator_gateway example)

```rust
// Current: Consumer must wire client manually, and it happens eagerly
pub async fn wire_client(hub: &ClientHub, resolver: &dyn DirectoryClient) -> Result<()> {
    let endpoint = resolver.resolve_grpc_service(SERVICE_NAME).await?;  // Fails if OoP not ready
    let client = CalculatorGrpcClient::connect(&endpoint.uri).await?;   // Fails if network issue
    hub.register::<dyn CalculatorClientV1>(Arc::new(client));
    Ok(())
}
```

## Proposed Solution

### Architecture Overview

```text
┌─────────────────────────────────────────────────────────────────────────┐
│                           SDK Crate (calculator-sdk)                    │
├─────────────────────────────────────────────────────────────────────────┤
│  CalculatorClientDescriptor                                             │
│    - MODULE_NAME: "calculator"                                          │
│    - SERVICE_NAME: "calculator.v1.CalculatorService"                    │
│    - Api: dyn CalculatorClientV1                                        │
│    - Availability: Optional (default)                                   │
├─────────────────────────────────────────────────────────────────────────┤
│  CalculatorGrpcClient (existing)                                        │
│    - Implements CalculatorClientV1                                      │
│    - Direct gRPC calls                                                  │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                           ModKit (libs/modkit)                          │
├─────────────────────────────────────────────────────────────────────────┤
│  GrpcClientProvider                                                     │
│    - Lazy resolution via DirectoryClient                                │
│    - Connection caching with eviction on error                          │
│    - Backoff/rate-limiting for reconnects                               │
│    - Transport middleware (timeouts, keepalive, tracing)                │
├─────────────────────────────────────────────────────────────────────────┤
│  LazyGrpcClient<D: GrpcClientDescriptor>                                │
│    - Generated wrapper implementing D::Api                              │
│    - Request-scoped middleware (SecurityContext propagation)            │
│    - Error mapping to SDK error types                                   │
├─────────────────────────────────────────────────────────────────────────┤
│  #[modkit::module] macro extension                                      │
│    - clients = [CalculatorClientDescriptor]                             │
│    - Auto-registers LazyGrpcClient into ClientHub                       │
│    - Auto-injects MODULE_NAME from each descriptor into deps            │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                      Consumer Module (calculator_gateway)               │
├─────────────────────────────────────────────────────────────────────────┤
│  #[modkit::module(                                                      │
│      name = "calculator_gateway",                                       │
│      capabilities = [rest],                                             │
│      clients = [calculator_sdk::CalculatorClientDescriptor],            │
│      // deps auto-injected: ["calculator"] from descriptor              │
│  )]                                                                     │
│                                                                         │
│  // No wire_client() call needed!                                       │
│  // Client is always available from ClientHub                           │
│  let calc = hub.get::<dyn CalculatorClientV1>()?;                       │
│  calc.add(ctx, a, b).await?;  // Lazy connect on first call             │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Detailed Implementation

### Phase 1: GrpcClientDescriptor Trait (SDK-side)

**Location**: `libs/modkit/src/clients/descriptor.rs`

```rust
//! Client descriptor traits for typed OoP client metadata.

use std::time::Duration;

/// Availability policy for OoP clients.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ClientAvailability {
    /// Client is optional; operations fail gracefully with SDK error (maps to HTTP 424).
    #[default]
    Optional,
    /// Client is required; module readiness may depend on availability.
    Required,
}

/// Configuration for gRPC client behavior.
#[derive(Debug, Clone)]
pub struct GrpcClientConfig {
    /// Connection timeout for initial connect.
    pub connect_timeout: Duration,
    /// Request timeout for individual RPC calls.
    pub request_timeout: Duration,
    /// Keepalive interval.
    pub keepalive_interval: Option<Duration>,
    /// Maximum backoff duration between reconnect attempts.
    pub max_backoff: Duration,
    /// Availability policy.
    pub availability: ClientAvailability,
}

impl Default for GrpcClientConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(30),
            keepalive_interval: Some(Duration::from_secs(30)),
            max_backoff: Duration::from_secs(60),
            availability: ClientAvailability::Optional,
        }
    }
}

/// Descriptor for a gRPC client, defined in SDK crates.
///
/// This trait binds compile-time type information to runtime metadata
/// needed for lazy client resolution and registration.
///
/// # Example
///
/// ```rust,ignore
/// pub struct CalculatorClientDescriptor;
///
/// impl GrpcClientDescriptor for CalculatorClientDescriptor {
///     type Api = dyn CalculatorClientV1;
///     type GrpcClient = CalculatorGrpcClient;
///
///     const MODULE_NAME: &'static str = "calculator";
///     const SERVICE_NAME: &'static str = "calculator.v1.CalculatorService";
///
///     fn config() -> GrpcClientConfig {
///         GrpcClientConfig::default()
///     }
/// }
/// ```
pub trait GrpcClientDescriptor: Send + Sync + 'static {
    /// The SDK API trait type (e.g., `dyn CalculatorClientV1`).
    type Api: ?Sized + Send + Sync + 'static;

    /// The concrete gRPC client type that implements `Api`.
    /// Must be constructible from a Channel.
    type GrpcClient: From<tonic::transport::Channel> + Send + Sync + 'static;

    /// Module name for dependency graph (used in `deps`).
    const MODULE_NAME: &'static str;

    /// gRPC service name for Directory resolution.
    const SERVICE_NAME: &'static str;

    /// Client configuration (timeouts, backoff, availability).
    fn config() -> GrpcClientConfig {
        GrpcClientConfig::default()
    }
}
```

### Phase 2: GrpcClientProvider (ModKit Core)

**Location**: `libs/modkit/src/clients/grpc_provider.rs`

```rust
//! Universal lazy gRPC client provider.
//!
//! Encapsulates endpoint resolution, connection management, caching,
//! and reconnection logic for any gRPC client.

use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::RwLock;
use tokio::sync::Semaphore;
use tonic::transport::Channel;

use crate::client_hub::ClientHub;
use crate::directory::DirectoryClient;
use crate::clients::descriptor::GrpcClientConfig;

/// Error type for provider operations.
#[derive(Debug, thiserror::Error)]
pub enum GrpcProviderError {
    #[error("service not found in directory: {service_name}")]
    ServiceNotFound { service_name: &'static str },

    #[error("connection failed: {0}")]
    ConnectionFailed(#[source] tonic::transport::Error),

    #[error("directory resolution failed: {0}")]
    DirectoryError(#[source] anyhow::Error),

    #[error("service temporarily unavailable (backoff active)")]
    Backoff { retry_after: Duration },
}

/// Cached connection state.
struct CachedConnection {
    channel: Channel,
    connected_at: Instant,
}

/// Internal state for the provider.
struct ProviderState {
    cached: Option<CachedConnection>,
    last_failure: Option<Instant>,
    failure_count: u32,
}

/// Universal lazy gRPC client provider.
///
/// This provider handles:
/// - Lazy endpoint resolution via DirectoryClient
/// - Connection caching with automatic eviction on errors
/// - Exponential backoff for reconnection attempts
/// - Rate limiting to prevent thundering herds
pub struct GrpcClientProvider {
    service_name: &'static str,
    config: GrpcClientConfig,
    hub: Arc<ClientHub>,
    state: RwLock<ProviderState>,
    connect_semaphore: Semaphore,
}

impl GrpcClientProvider {
    /// Create a new provider for the given service.
    pub fn new(
        service_name: &'static str,
        config: GrpcClientConfig,
        hub: Arc<ClientHub>,
    ) -> Self {
        Self {
            service_name,
            config,
            hub,
            state: RwLock::new(ProviderState {
                cached: None,
                last_failure: None,
                failure_count: 0,
            }),
            connect_semaphore: Semaphore::new(1),
        }
    }

    /// Get a connected channel, resolving and connecting lazily.
    ///
    /// Returns a cached channel if available, otherwise resolves the endpoint
    /// and establishes a new connection.
    pub async fn get_channel(&self) -> Result<Channel, GrpcProviderError> {
        // Fast path: return cached connection
        {
            let state = self.state.read();
            if let Some(ref cached) = state.cached {
                return Ok(cached.channel.clone());
            }

            // Check backoff
            if let Some(last_failure) = state.last_failure {
                let backoff = self.calculate_backoff(state.failure_count);
                let elapsed = last_failure.elapsed();
                if elapsed < backoff {
                    return Err(GrpcProviderError::Backoff {
                        retry_after: backoff - elapsed,
                    });
                }
            }
        }

        // Slow path: acquire semaphore and connect
        let _permit = self.connect_semaphore.acquire().await
            .expect("semaphore is never closed");

        // Double-check after acquiring semaphore
        {
            let state = self.state.read();
            if let Some(ref cached) = state.cached {
                return Ok(cached.channel.clone());
            }
        }

        // Resolve and connect
        self.connect_internal().await
    }

    /// Evict the cached connection (call on transport errors).
    pub fn evict(&self) {
        let mut state = self.state.write();
        state.cached = None;
        state.last_failure = Some(Instant::now());
        state.failure_count = state.failure_count.saturating_add(1);
        tracing::warn!(
            service = self.service_name,
            failure_count = state.failure_count,
            "Evicted cached gRPC connection"
        );
    }

    /// Reset failure state (call on successful RPC).
    pub fn reset_failures(&self) {
        let mut state = self.state.write();
        if state.failure_count > 0 {
            state.failure_count = 0;
            state.last_failure = None;
            tracing::debug!(service = self.service_name, "Reset failure state after success");
        }
    }

    async fn connect_internal(&self) -> Result<Channel, GrpcProviderError> {
        // Resolve endpoint from DirectoryClient
        let directory = self
            .hub
            .get::<dyn DirectoryClient>()
            .map_err(|e| GrpcProviderError::DirectoryError(e.into()))?;

        let endpoint = directory
            .resolve_grpc_service(self.service_name)
            .await
            .map_err(|e| GrpcProviderError::DirectoryError(e))?;

        tracing::debug!(
            service = self.service_name,
            uri = %endpoint.uri,
            "Resolved service endpoint"
        );

        // Connect with configured timeouts
        let channel = Channel::from_shared(endpoint.uri.clone())
            .map_err(GrpcProviderError::ConnectionFailed)?
            .connect_timeout(self.config.connect_timeout)
            .timeout(self.config.request_timeout);

        let channel = if let Some(keepalive) = self.config.keepalive_interval {
            channel
                .http2_keep_alive_interval(keepalive)
                .keep_alive_timeout(Duration::from_secs(20))
        } else {
            channel
        };

        let channel = channel
            .connect()
            .await
            .map_err(GrpcProviderError::ConnectionFailed)?;

        // Cache the connection
        {
            let mut state = self.state.write();
            state.cached = Some(CachedConnection {
                channel: channel.clone(),
                connected_at: Instant::now(),
            });
            state.failure_count = 0;
            state.last_failure = None;
        }

        tracing::info!(
            service = self.service_name,
            uri = %endpoint.uri,
            "Established gRPC connection"
        );

        Ok(channel)
    }

    fn calculate_backoff(&self, failure_count: u32) -> Duration {
        let base = Duration::from_millis(100);
        let max = self.config.max_backoff;
        let backoff = base.saturating_mul(2u32.saturating_pow(failure_count.min(10)));
        backoff.min(max)
    }
}
```

### Phase 3: LazyGrpcClient Wrapper (Generated or Manual)

**Location**: `libs/modkit/src/clients/lazy_client.rs`

For the initial implementation, we provide a manual wrapper pattern. The macro generation can be added later.

```rust
//! Lazy gRPC client wrapper that implements SDK traits.
//!
//! This wrapper delegates to GrpcClientProvider for connection management
//! and handles SecurityContext propagation.

use std::sync::Arc;

use crate::clients::grpc_provider::{GrpcClientProvider, GrpcProviderError};
use crate::clients::descriptor::GrpcClientDescriptor;

/// Error returned by lazy clients when the OoP dependency is unavailable.
#[derive(Debug, thiserror::Error)]
pub enum LazyClientError {
    #[error("service unavailable: {service_name}")]
    Unavailable {
        service_name: &'static str,
        #[source]
        source: GrpcProviderError,
    },

    #[error("RPC failed: {0}")]
    RpcFailed(#[source] tonic::Status),
}

impl LazyClientError {
    /// Returns true if this error indicates the service is temporarily unavailable.
    /// REST handlers should map this to HTTP 424 Failed Dependency.
    pub fn is_dependency_unavailable(&self) -> bool {
        matches!(self, LazyClientError::Unavailable { .. })
    }
}

/// Macro to generate a lazy client wrapper for a GrpcClientDescriptor.
///
/// This generates a struct that:
/// - Implements the SDK API trait
/// - Uses GrpcClientProvider for lazy connection management
/// - Propagates SecurityContext via gRPC metadata
/// - Maps transport errors to SDK error types
///
/// # Example
///
/// ```rust,ignore
/// modkit::lazy_grpc_client! {
///     /// Lazy client for Calculator service.
///     pub struct LazyCalculatorClient for calculator_sdk::CalculatorClientDescriptor {
///         // Method implementations are generated based on the trait
///     }
/// }
/// ```
#[macro_export]
macro_rules! lazy_grpc_client {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident for $descriptor:ty {
            $(
                async fn $method:ident(
                    &self,
                    ctx: &SecurityContext
                    $(, $arg:ident : $arg_ty:ty)*
                ) -> Result<$ret:ty, $err:ty> $body:block
            )*
        }
    ) => {
        $(#[$meta])*
        $vis struct $name {
            provider: std::sync::Arc<$crate::clients::grpc_provider::GrpcClientProvider>,
        }

        impl $name {
            pub fn new(provider: std::sync::Arc<$crate::clients::grpc_provider::GrpcClientProvider>) -> Self {
                Self { provider }
            }
        }

        // Trait implementation would be generated here
        // For now, this is a placeholder showing the pattern
    };
}
```

### Phase 4: SDK Crate Updates (calculator-sdk example)

**Location**: `examples/oop-modules/calculator/calculator-sdk/src/descriptor.rs`

```rust
//! Client descriptor for Calculator SDK.

use modkit::clients::descriptor::{GrpcClientDescriptor, GrpcClientConfig, ClientAvailability};
use crate::api::CalculatorClientV1;
use crate::client::CalculatorGrpcClient;

/// Descriptor for the Calculator gRPC client.
///
/// Used by consumer modules to declare their dependency on the Calculator service
/// via `#[modkit::module(clients = [CalculatorClientDescriptor])]`.
pub struct CalculatorClientDescriptor;

impl GrpcClientDescriptor for CalculatorClientDescriptor {
    type Api = dyn CalculatorClientV1;
    type GrpcClient = CalculatorGrpcClient;

    const MODULE_NAME: &'static str = "calculator";
    const SERVICE_NAME: &'static str = crate::SERVICE_NAME;

    fn config() -> GrpcClientConfig {
        GrpcClientConfig {
            availability: ClientAvailability::Optional,
            ..Default::default()
        }
    }
}
```

**Updated SDK lib.rs**:

```rust
// ... existing exports ...

// Client descriptor for lazy wiring
mod descriptor;
pub use descriptor::CalculatorClientDescriptor;
```

### Phase 5: Lazy Client Implementation for Calculator

**Location**: `examples/oop-modules/calculator/calculator-sdk/src/lazy_client.rs`

```rust
//! Lazy gRPC client for Calculator service.
//!
//! This client wraps GrpcClientProvider and implements CalculatorClientV1,
//! providing lazy connection management and graceful degradation.

use std::sync::Arc;

use async_trait::async_trait;
use modkit::clients::grpc_provider::GrpcClientProvider;
use modkit_security::SecurityContext;
use tonic::metadata::MetadataMap;

use crate::api::{CalculatorClientV1, CalculatorError};
use crate::client::CalculatorGrpcClient;
use crate::proto;

/// Lazy client for Calculator service.
///
/// This client:
/// - Resolves the Calculator endpoint lazily on first call
/// - Caches the connection for subsequent calls
/// - Handles reconnection with backoff on failures
/// - Propagates SecurityContext via gRPC metadata
pub struct LazyCalculatorClient {
    provider: Arc<GrpcClientProvider>,
}

impl LazyCalculatorClient {
    /// Create a new lazy client with the given provider.
    pub fn new(provider: Arc<GrpcClientProvider>) -> Self {
        Self { provider }
    }
}

#[async_trait]
impl CalculatorClientV1 for LazyCalculatorClient {
    async fn add(&self, ctx: &SecurityContext, a: i64, b: i64) -> Result<i64, CalculatorError> {
        // Get channel (lazy connect)
        let channel = self.provider.get_channel().await.map_err(|e| {
            tracing::warn!(error = %e, "Calculator service unavailable");
            CalculatorError::Unavailable {
                message: format!("Calculator service unavailable: {}", e),
            }
        })?;

        // Create the gRPC client using the descriptor's GrpcClient type.
        // CalculatorGrpcClient wraps the proto client and implements From<Channel>.
        let client = CalculatorGrpcClient::from(channel);

        // Propagate security context via metadata
        let mut request = tonic::Request::new(proto::AddRequest { a, b });
        inject_security_context(ctx, request.metadata_mut());

        // Make the call
        let response = client.add(request).await.map_err(|status| {
            // Evict connection on transport errors
            if is_transport_error(&status) {
                self.provider.evict();
            }
            map_status_to_error(status)
        })?;

        // Reset failure state on success
        self.provider.reset_failures();

        Ok(response.into_inner().result)
    }

    async fn subtract(&self, ctx: &SecurityContext, a: i64, b: i64) -> Result<i64, CalculatorError> {
        let channel = self.provider.get_channel().await.map_err(|e| {
            CalculatorError::Unavailable {
                message: format!("Calculator service unavailable: {}", e),
            }
        })?;

        let client = CalculatorGrpcClient::from(channel);
        let mut request = tonic::Request::new(proto::SubtractRequest { a, b });
        inject_security_context(ctx, request.metadata_mut());

        let response = client.subtract(request).await.map_err(|status| {
            if is_transport_error(&status) {
                self.provider.evict();
            }
            map_status_to_error(status)
        })?;

        self.provider.reset_failures();
        Ok(response.into_inner().result)
    }

    // ... other methods follow the same pattern ...
}

/// Inject SecurityContext into gRPC metadata.
fn inject_security_context(ctx: &SecurityContext, metadata: &mut MetadataMap) {
    if let Some(tenant_id) = ctx.tenant_id() {
        if let Ok(value) = tenant_id.to_string().parse() {
            metadata.insert("x-tenant-id", value);
        }
    }
    if let Some(user_id) = ctx.user_id() {
        if let Ok(value) = user_id.to_string().parse() {
            metadata.insert("x-user-id", value);
        }
    }
    // Add other context fields as needed
}

/// Check if the error is a transport-level error (connection lost, etc.)
fn is_transport_error(status: &tonic::Status) -> bool {
    matches!(
        status.code(),
        tonic::Code::Unavailable | tonic::Code::Unknown | tonic::Code::Internal
    )
}

/// Map tonic Status to SDK error type.
fn map_status_to_error(status: tonic::Status) -> CalculatorError {
    match status.code() {
        tonic::Code::InvalidArgument => CalculatorError::InvalidArgument {
            message: status.message().to_string(),
        },
        tonic::Code::NotFound => CalculatorError::NotFound {
            message: status.message().to_string(),
        },
        tonic::Code::Unavailable => CalculatorError::Unavailable {
            message: status.message().to_string(),
        },
        _ => CalculatorError::Internal {
            message: status.message().to_string(),
        },
    }
}
```

### Phase 6: Module Macro Extension

**Location**: `libs/modkit-macros/src/module.rs` (extension)

The `#[modkit::module]` macro will be extended to support `clients = [...]`:

```rust
// Conceptual macro expansion for:
// #[modkit::module(
//     name = "calculator_gateway",
//     capabilities = [rest],
//     clients = [calculator_sdk::CalculatorClientDescriptor],
//     // Note: deps is auto-injected from clients; no need to specify manually.
// )]

// Generated code (simplified):
impl CalculatorGateway {
    fn __register_lazy_clients(ctx: &ModuleCtx) -> anyhow::Result<()> {
        use modkit::clients::descriptor::GrpcClientDescriptor;

        // For each descriptor in `clients`:
        {
            type D = calculator_sdk::CalculatorClientDescriptor;
            let config = D::config();
            let provider = std::sync::Arc::new(
                modkit::clients::grpc_provider::GrpcClientProvider::new(
                    D::SERVICE_NAME,
                    config,
                    ctx.client_hub(),
                )
            );
            let lazy_client: std::sync::Arc<<D as GrpcClientDescriptor>::Api> =
                std::sync::Arc::new(calculator_sdk::LazyCalculatorClient::new(provider));
            ctx.client_hub().register::<<D as GrpcClientDescriptor>::Api>(lazy_client);
        }

        Ok(())
    }
}

// The macro also ensures deps includes the module_name from each descriptor
// (validated at compile time or augmented automatically)
```

### Phase 7: Registry Extension for Soft OoP Deps

**Location**: `libs/modkit/src/registry.rs` (extension)

```rust
/// Extended dependency resolution for OoP modules.
impl ModuleRegistry {
    /// Resolve dependencies, treating unknown deps as potential OoP soft deps.
    ///
    /// - If dep is a registered core module → hard dep (topo-sort)
    /// - If dep is configured as `runtime.type = oop` → soft dep (no topo-sort)
    /// - Otherwise → error (unknown dependency)
    pub fn resolve_dependencies_with_oop(
        &self,
        module_name: &str,
        deps: &[&str],
        config: &AppConfig,
    ) -> Result<ResolvedDeps, RegistryError> {
        let mut hard_deps = Vec::new();
        let mut soft_deps = Vec::new();

        for dep in deps {
            if self.has_module(dep) {
                // Known in-process module
                hard_deps.push(*dep);
            } else if config.is_oop_module(dep) {
                // OoP module declared in config
                soft_deps.push(*dep);
            } else {
                return Err(RegistryError::UnknownDependency {
                    module: module_name.to_string(),
                    dependency: dep.to_string(),
                });
            }
        }

        Ok(ResolvedDeps { hard_deps, soft_deps })
    }
}

/// Result of dependency resolution.
pub struct ResolvedDeps {
    /// Hard dependencies (in-process, participate in topo-sort).
    pub hard_deps: Vec<&'static str>,
    /// Soft dependencies (OoP, do not block startup).
    pub soft_deps: Vec<&'static str>,
}
```

---

## Consumer Module Changes (calculator_gateway)

### Before (current pattern)

```rust
#[modkit::module(
    name = "calculator_gateway",
    capabilities = [rest],
    deps = ["calculator"]
)]
pub struct CalculatorGateway;

#[async_trait]
impl modkit::Module for CalculatorGateway {
    async fn init(&self, ctx: &ModuleCtx) -> Result<()> {
        // Must wire client manually - FAILS if calculator not ready
        let directory = ctx.client_hub().get::<dyn DirectoryClient>()?;
        calculator_sdk::wire_client(ctx.client_hub(), &*directory).await?;
        // ...
    }
}
```

### After (proposed pattern)

```rust
#[modkit::module(
    name = "calculator_gateway",
    capabilities = [rest],
    // deps is auto-injected from clients: the macro reads MODULE_NAME
    // from each descriptor and adds it as a soft OoP dep.
    clients = [calculator_sdk::CalculatorClientDescriptor],
)]
pub struct CalculatorGateway;

#[async_trait]
impl modkit::Module for CalculatorGateway {
    async fn init(&self, ctx: &ModuleCtx) -> Result<()> {
        // No wire_client() needed!
        // LazyCalculatorClient is auto-registered by the macro.
        let service = Arc::new(Service::new(ctx.client_hub()));
        ctx.client_hub().register::<Service>(service);
        Ok(())
    }
}

// In domain service - unchanged!
impl Service {
    pub async fn add(&self, ctx: &SecurityContext, a: i64, b: i64) -> Result<i64, ServiceError> {
        let calculator = self.client_hub.get::<dyn CalculatorClientV1>()?;
        // Lazy connect happens here on first call
        calculator.add(ctx, a, b).await.map_err(|e| {
            // CalculatorError::Unavailable maps to HTTP 424
            ServiceError::RemoteError(e.to_string())
        })
    }
}
```

---

## Error Handling and HTTP 424

The lazy client returns `CalculatorError::Unavailable` when the OoP service cannot be reached. REST handlers should map this to HTTP 424 Failed Dependency:

```rust
// In REST handler error mapping
// Use a typed error variant instead of string matching
impl From<ServiceError> for Problem {
    fn from(err: ServiceError) -> Self {
        match err {
            ServiceError::DependencyUnavailable { service, source } => {
                Problem::failed_dependency()
                    .with_detail(format!("{} unavailable: {}", service, source))
            }
            ServiceError::RemoteError(msg) => {
                Problem::bad_gateway()
                    .with_detail(msg)
            }
            ServiceError::Internal(msg) => {
                Problem::internal_server_error()
                    .with_detail(msg)
            }
        }
    }
}

// ServiceError should have a typed variant for dependency failures:
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("dependency unavailable: {service}")]
    DependencyUnavailable {
        service: &'static str,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[error("remote error: {0}")]
    RemoteError(String),
    #[error("internal error: {0}")]
    Internal(String),
}

// In domain service, map SDK errors to the typed variant:
impl Service {
    pub async fn add(&self, ctx: &SecurityContext, a: i64, b: i64) -> Result<i64, ServiceError> {
        let calculator = self.client_hub.get::<dyn CalculatorClientV1>()?;
        calculator.add(ctx, a, b).await.map_err(|e| match e {
            CalculatorError::Unavailable { message } => ServiceError::DependencyUnavailable {
                service: "calculator",
                source: message.into(),
            },
            other => ServiceError::RemoteError(other.to_string()),
        })
    }
}
```

---

## Implementation Phases

### Phase 1: Core Infrastructure (Week 1-2)
1. Add `GrpcClientDescriptor` trait to `libs/modkit/src/clients/descriptor.rs`
2. Implement `GrpcClientProvider` in `libs/modkit/src/clients/grpc_provider.rs`
3. Add `LazyClientError` type and helper utilities
4. Unit tests for provider (mock DirectoryClient)

### Phase 2: SDK Updates (Week 2-3)
1. Add `CalculatorClientDescriptor` to calculator-sdk
2. Implement `LazyCalculatorClient` manually
3. Update calculator-sdk exports
4. Integration tests with mock gRPC server

### Phase 3: Macro Extension (Week 3-4)
1. Extend `#[modkit::module]` to parse `clients = [...]`
2. Generate lazy client registration code
3. Auto-augment `deps` with module names from descriptors
4. Compile-time validation of descriptor types

### Phase 4: Registry Extension (Week 4)
1. Implement soft OoP dep resolution in registry
2. Update topo-sort to exclude soft deps
3. Add config validation for OoP modules
4. Integration tests for startup ordering

### Phase 5: Migration & Documentation (Week 5)
1. Update calculator_gateway example
2. Update `docs/modkit_unified_system/09_oop_grpc_sdk_pattern.md` (primary migration doc)
3. Add migration guide for existing SDKs to `09_oop_grpc_sdk_pattern.md`
4. Update checklists

---

## Testing Strategy

### Unit Tests
- `GrpcClientProvider`: connection caching, backoff, eviction
- `LazyCalculatorClient`: error mapping, context propagation
- Registry: soft dep resolution

### Integration Tests
- Startup with unavailable OoP → module starts successfully
- First call triggers lazy connect
- Connection failure → backoff → retry
- Successful call → failure state reset

### E2E Tests
- calculator_gateway starts without calculator OoP
- REST call returns 424 when calculator unavailable
- REST call succeeds after calculator becomes available

---

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Macro complexity | Start with manual lazy client impl; add codegen later |
| Breaking existing SDKs | Backward-compatible: `wire_client()` still works |
| Performance overhead | Provider uses fast-path caching; no overhead on hot path |
| Debugging difficulty | Detailed tracing in provider and lazy client |

---

## Success Criteria

1. **No eager wiring**: Consumer modules do not call `wire_client()` in `init()`
2. **Graceful startup**: Modules start even if OoP dependencies are unavailable
3. **Per-operation degradation**: Missing OoP → HTTP 424 for affected endpoints only
4. **Single source of truth**: `clients = [...]` declares all OoP dependencies
5. **Consistent behavior**: All gRPC clients use the same provider infrastructure

---

## Appendix: File Structure

```text
libs/modkit/src/
├── clients/
│   ├── mod.rs              # Module exports
│   ├── descriptor.rs       # GrpcClientDescriptor trait
│   ├── grpc_provider.rs    # GrpcClientProvider implementation
│   └── lazy_client.rs      # LazyClientError and utilities
├── lib.rs                  # Add `pub mod clients;`
└── ...

libs/modkit-macros/src/
├── module.rs               # Extended to parse `clients = [...]`
└── ...

examples/oop-modules/calculator/calculator-sdk/src/
├── descriptor.rs           # CalculatorClientDescriptor
├── lazy_client.rs          # LazyCalculatorClient
├── lib.rs                  # Updated exports
└── ...
```
