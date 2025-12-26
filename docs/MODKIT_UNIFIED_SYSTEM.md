# ModKit — Architecture & Developer Guide (DDD-light)

This guide explains how to build production-grade modules on **ModKit**: how to lay out a module, declare it with a
macro, wire REST with a type-safe builder, publish typed clients, and run background services with a clean lifecycle. It
also describes the DDD-light layering and conventions used across modules.

---

## What ModKit provides

* **Composable modules** discovered via `inventory`, initialized in dependency order.
* **Ingress as a module** (e.g., `api_ingress`) that owns the Axum router and OpenAPI document.
* **Type-safe REST** via an operation builder that prevents half-wired routes at compile time.
* **Server-Sent Events (SSE)** with type-safe broadcasters and domain event integration.
* **OpenAPI 3.1** generation using `utoipa` with automatic schema registration for DTOs.
* **Standardized HTTP errors** with RFC-9457 `Problem` (implements `IntoResponse` directly).
* **Typed ClientHub** for in-process clients (resolve by interface type + optional scope).
* **Plugin architecture** via scoped ClientHub registration and GTS-based discovery (see [MODKIT_PLUGINS.md](./MODKIT_PLUGINS.md)).
* **Lifecycle** helpers and wrappers for long-running tasks and graceful shutdown.
* **Lock-free hot paths** via atomic `Arc` swaps for read-mostly state.

---

## Canonical layout (DDD-light)

Place each module under `modules/<name>/`:

```
modules/<name>/
  ├─ src/
  │  ├─ lib.rs                       # module declaration, exports
  │  ├─ module.rs                    # main struct + Module/Db/Rest/Stateful impls
  │  ├─ config.rs                    # typed config (optional)
  │  ├─ api/
  │  │  └─ rest/
  │  │     ├─ dto.rs                 # HTTP DTOs (serde/utoipa) — REST-only types
  │  │     ├─ handlers.rs            # Axum handlers (web controllers)
  │  │     └─ routes.rs              # route & OpenAPI registration (OperationBuilder)
  │  ├─ contract/                    # public API surface (for other modules)
  │  │  ├─ mod.rs
  │  │  ├─ client.rs                 # traits for ClientHub and DTOs
  │  │  ├─ model.rs                  # DTOs exposed to other modules (no REST specifics)
  │  │  └─ error.rs
  │  ├─ domain/                      # internal business logic
  │  │  ├─ mod.rs
  │  │  ├─ model.rs                  # rich domain models
  │  │  ├─ error.rs
  │  │  └─ service.rs                # orchestration/business rules
  │  └─ infra/                       # “low-level”: DB, system, IO, adapters
  │     ├─ storage/
  │     │  ├─ entity.rs              # e.g., SeaORM entities / SQL mappings
  │     │  ├─ mapper.rs              # entity <-> contract conversions (From impls)
  │     │  └─ migrations/
  │     │     ├─ mod.rs
  │     │     └─ initial_001.rs
  │     └─ (other platform adapters)
  ├─ spec/
  │  └─ proto/                       # proto files (if present)
  └─ Cargo.toml
```

Notes:

* Handlers may call `domain::service` directly.
* For simple internal modules you may re-export domain models via `contract::model`.
* Gateways host client implementations (e.g., local). Traits & DTOs live in `contract`.
* Infra may use SeaORM or raw SQL (SQLx or your choice).

---

## ModuleCtx (what you get at runtime)

```rust
pub trait ConfigProvider: Send + Sync {
    /// Returns raw JSON section for the module, if any.
    fn get_module_config(&self, module_name: &str) -> Option<&serde_json::Value>;
}

#[derive(Clone)]
pub struct ModuleCtx {
    pub(crate) db: Option<std::sync::Arc<db::DbHandle>>,
    pub(crate) config_provider: Option<std::sync::Arc<dyn ConfigProvider>>,
    pub(crate) client_hub: std::sync::Arc<crate::client_hub::ClientHub>,
    pub(crate) cancellation_token: tokio_util::sync::CancellationToken,
    pub(crate) module_name: Option<std::sync::Arc<str>>,
    pub(crate) instance_id: uuid::Uuid,  // Process-level unique instance ID
}
```

**Note:** The `instance_id` is generated once at process startup and shared by all modules in the same process. OoP
modules receive their own unique instance ID.

### Common usage

**Typed config**

```rust
#[derive(serde::Deserialize, Default, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct MyModuleConfig {
    /* fields */
}
```

**DB access (SeaORM / SQLx)**

```rust
let sea = db.sea();      // SeaORM connection
let pool = db.sqlx_pool();  // SQLx pool
```

**Clients (publish & consume)**

```rust
// publish (provider module, in init()):
expose_my_module_client( & ctx, & api) ?;

// consume (consumer module, in init()):
let api = my_module_client( & ctx.client_hub);
// or without helpers:
let api = ctx.client_hub.get::<dyn my_module::contract::client::MyModuleApi>() ?;
```

**Cancellation**

```rust
let child = ctx.cancellation_token.child_token();
// pass `child` into background tasks for cooperative shutdown
```

---

## Declarative module registration — `#[modkit::module(...)]`

Attach the attribute to your main struct. The macro:

* Adds inventory entry for auto-discovery.
* Registers **name**, **deps**, **caps** (capabilities).
* Instantiates via `ctor = <expr>` or `Default` if `ctor` is omitted.
* Optionally emits **ClientHub** helpers.
* Optionally wires **lifecycle** when you add `lifecycle(...)`.

### Full syntax

```rust
#[modkit::module(
    name = "my_module",
    deps = ["foo", "bar"], // api_ingress dependency will be added automatically for rest module capability
    capabilities = [db, rest, stateful, /* rest_host if you own the HTTP server */],
    client = contract::client::MyModuleApi,
    ctor = MyModule::new(),
    lifecycle(entry = "serve", stop_timeout = "30s", await_ready)
)]
pub struct MyModule {
    /* fields */
}
```

### Capabilities

* `db` → implement `DbModule` (migrations / schema setup).
* `rest` → implement `RestfulModule` (register routes synchronously).
* `rest_host` → own the Axum server/OpenAPI (e.g., `api_ingress`).
* `stateful` → background job:

    * With `lifecycle(...)`, the macro generates `Runnable` and registers `WithLifecycle<Self>`.
    * Without it, implement `StatefulModule` yourself.

### Client helpers (when `client` is set)

Generated helpers:

* `expose_<module>_client(ctx, &Arc<dyn Trait>) -> anyhow::Result<()>`
* `expose_<module>_client_in(ctx, scope: &str, &Arc<dyn Trait>) -> anyhow::Result<()>`
* `<module>_client(hub: &ClientHub) -> Arc<dyn Trait>`
* `<module>_client_in(hub: &ClientHub, scope: &str) -> Arc<dyn Trait>`

---

## Lifecycle — macro attributes & state machine

`WithLifecycle<T>` provides a ready-to-use lifecycle with cancellation semantics.

```rust
#[modkit::module(
    name = "api_ingress",
    capabilities = [rest_host, rest, stateful],
    lifecycle(entry = "serve", stop_timeout = "30s", await_ready)
)]
pub struct ApiIngress {
    /* ... */
}

impl ApiIngress {
    // accepted signatures:
    // 1) async fn serve(self: Arc<Self>, cancel: CancellationToken) -> Result<()>
    // 2) async fn serve(self: Arc<Self>, cancel: CancellationToken, ready: ReadySignal) -> Result<()>
    async fn serve(
        self: std::sync::Arc<Self>,
        cancel: tokio_util::sync::CancellationToken,
        ready: modkit::lifecycle::ReadySignal
    ) -> anyhow::Result<()> {
        // bind sockets/resources before flipping to Running
        ready.notify();
        cancel.cancelled().await;
        Ok(())
    }
}
```

**States & transitions**

```
Stopped ── start() ── Starting ──(await_ready? then ready.notify())──▶ Running
   ▲                                  │
   │                                  └─ if await_ready = false → Running immediately
   └──────────── stop()/cancel ────────────────────────────────────────────────┘
```

`WithLifecycle::stop()` waits up to `stop_timeout`, then aborts the task if needed.

---

## Out-of-Process Modules (OoP)

ModKit supports running modules as separate processes with gRPC-based inter-process communication. This enables:

* **Process isolation** — modules run in separate processes for fault isolation
* **Language flexibility** — OoP modules can be implemented in any language (with gRPC support)
* **Independent scaling** — modules can be scaled independently
* **Gradual migration** — existing modules can be moved out-of-process without code changes

### RuntimeKind

Modules can run in two modes:

```rust
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeKind {
    #[default]
    Local,  // In-process (default)
    Oop,    // Out-of-process
}
```

### OoP Module Configuration

Configure OoP modules in your YAML config:

```yaml
modules:
  calculator:
    runtime:
      type: oop
      execution:
        executable_path: "~/.hyperspot/bin/calculator-oop.exe"
        args: [ ]
        working_directory: null
        environment:
          RUST_LOG: "info"
    config:
      some_setting: "value"
```

**Configuration fields:**

* `type: oop` — marks the module as out-of-process
* `executable_path` — path to the module binary (supports `~` expansion)
* `args` — command-line arguments passed to the executable
* `working_directory` — optional working directory for the process
* `environment` — environment variables to set for the process

### OoP Bootstrap Library

Use `modkit::bootstrap::oop` to bootstrap OoP modules (remember to enable the bootstrap feature from modkit):

```rust
use modkit::bootstrap::oop::{OopRunOptions, run_oop_with_options};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opts = OopRunOptions {
        module_name: "my_module".to_string(),
        instance_id: None,  // Auto-generated UUID
        directory_endpoint: "http://127.0.0.1:50051".to_string(),
        config_path: None,
        verbose: 0,
        print_config: false,
        heartbeat_interval_secs: 5,
    };

    run_oop_with_options(opts).await
}
```

**`OopRunOptions` fields:**

| Field                     | Description                                         |
|---------------------------|-----------------------------------------------------|
| `module_name`             | Logical module name (e.g., "file_parser")           |
| `instance_id`             | Instance ID (defaults to random UUID)               |
| `directory_endpoint`      | DirectoryService gRPC endpoint                      |
| `config_path`             | Path to configuration file                          |
| `verbose`                 | Log verbosity (0=default, 1=info, 2=debug, 3=trace) |
| `print_config`            | Print effective config and exit                     |
| `heartbeat_interval_secs` | Heartbeat interval (default: 5)                     |

### OoP Lifecycle

When an OoP module starts:

1. **Configuration loading** — loads config from file or `MODKIT_MODULE_CONFIG` env var
2. **Logging initialization** — sets up tracing with optional OTEL
3. **DirectoryService connection** — connects to the master host's directory service
4. **Instance registration** — registers with DirectoryService for discovery
5. **Heartbeat loop** — starts background heartbeat task
6. **Module lifecycle** — runs the normal module lifecycle (init → migrate → start)
7. **Graceful shutdown** — deregisters from DirectoryService on exit

### Shutdown Model

Shutdown is driven by a single root `CancellationToken` per process:

* OS signals (SIGTERM, SIGINT, Ctrl+C) are hooked at bootstrap level
* The root token is passed to `RunOptions::Token` for module runtime shutdown
* Background tasks (like heartbeat) use child tokens derived from the root
* On shutdown, the module deregisters itself from DirectoryService before exiting

---

## Module Orchestrator & Directory API

The **Module Orchestrator** provides service discovery and instance management for both in-process and OoP modules.

### DirectoryApi Trait

```rust
#[async_trait]
pub trait DirectoryApi: Send + Sync {
    /// Resolve a gRPC service by its logical name to an endpoint
    async fn resolve_grpc_service(&self, service_name: &str) -> Result<ServiceEndpoint>;

    /// List all service instances for a given module
    async fn list_instances(&self, module: &str) -> Result<Vec<ServiceInstanceInfo>>;

    /// Register a new module instance with the directory
    async fn register_instance(&self, info: RegisterInstanceInfo) -> Result<()>;

    /// Deregister a module instance (for graceful shutdown)
    async fn deregister_instance(&self, module: &str, instance_id: &str) -> Result<()>;

    /// Send a heartbeat for a module instance to indicate it's still alive
    async fn send_heartbeat(&self, module: &str, instance_id: &str) -> Result<()>;
}
```

### Service Endpoint Types

```rust
/// Represents an endpoint where a service can be reached
pub struct ServiceEndpoint {
    pub uri: String,
}

impl ServiceEndpoint {
    pub fn tcp(host: &str, port: u16) -> Self;  // "http://host:port"
    pub fn uds(path: impl AsRef<Path>) -> Self; // "unix:///path/to/socket"
}

/// Information about a service instance
pub struct ServiceInstanceInfo {
    pub module: String,
    pub instance_id: String,
    pub endpoint: ServiceEndpoint,
    pub version: Option<String>,
}

/// Information for registering a new module instance
pub struct RegisterInstanceInfo {
    pub module: String,
    pub instance_id: String,
    pub grpc_services: Vec<(String, ServiceEndpoint)>,
    pub version: Option<String>,
}
```

### Using DirectoryApi

**From an OoP module** — the DirectoryApi client is injected into the ClientHub:

```rust
async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
    // DirectoryApi is available via ClientHub in OoP modules
    let directory = ctx.client_hub.get::<dyn DirectoryApi>()?;

    // Resolve another service's endpoint
    let endpoint = directory.resolve_grpc_service("my.package.MyService").await?;

    Ok(())
}
```

**From the master host** — the Module Orchestrator registers itself as the DirectoryApi implementation.

---

## Backends

Backends manage the lifecycle of out-of-process module instances.

### BackendKind

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    LocalProcess,  // Spawn as local child process
    K8s,           // Deploy to Kubernetes (future)
    Static,        // Pre-configured static endpoints
    Mock,          // For testing
}
```

### OopBackend Trait

```rust
#[async_trait]
pub trait OopBackend: Send + Sync {
    /// Spawn an OoP module instance
    async fn spawn(&self, config: OopSpawnConfig) -> Result<()>;

    /// Shutdown all spawned instances
    async fn shutdown_all(&self);
}
```

### LocalProcessBackend

Spawns modules as local child processes with managed lifecycle:

```rust
use modkit::LocalProcessBackend;
use tokio_util::sync::CancellationToken;

let cancel = CancellationToken::new();
let backend = LocalProcessBackend::new(cancel.clone());

// Backend automatically:
// - Spawns child processes with proper environment
// - Forwards stdout/stderr to parent process logs
// - Gracefully stops all processes when token is cancelled
// - Force-kills processes after 5-second grace period
```

**Features:**

* **CancellationToken integration** — coordinated shutdown across all processes
* **Log forwarding** — child process output forwarded with module context
* **Process group management** — clean termination of process trees
* **Graceful shutdown** — 5-second grace period before force-kill

### Log Forwarding

Child process output is captured and re-emitted with module context:

```rust
// Child process logs appear as:
// INFO calculator: Starting service...
// DEBUG calculator: Handling request id=123
```

---

## SDK Pattern for Inter-Module Communication

For OoP modules, use the **SDK pattern** to define typed APIs that work across process boundaries.
The SDK crate combines API traits, types, gRPC stubs, client implementation, and wiring helpers.

### Module Structure with SDK

```
modules/my_module/
  ├── my_module-sdk/              # SDK for consumers (everything in one place)
  │   ├── Cargo.toml
  │   ├── build.rs                # Proto compilation
  │   ├── proto/
  │   │   └── my_module.proto     # gRPC service definition
  │   └── src/
  │       ├── lib.rs              # Re-exports everything
  │       ├── api.rs              # API trait + types + errors
  │       ├── client.rs           # gRPC client impl (using modkit-transport-grpc)
  │       └── wiring.rs           # wire_client() helper function
  └── my_module/                  # Module implementation + SERVER
      ├── Cargo.toml
      └── src/
          ├── lib.rs              # Module definition, re-exports SDK
          ├── module.rs           # Module struct + traits
          ├── grpc_server.rs      # gRPC server implementation
          └── main.rs             # OoP binary entry point
```

**Key points:**

- The `-sdk` crate contains everything consumers need: API trait, types, gRPC client, and wiring helpers
- Server implementations are owned by the module itself, not the SDK
- Consumers only need one dependency: `my_module-sdk`

### SDK Crate Structure

```rust
// my_module-sdk/src/lib.rs
#![forbid(unsafe_code)]

// API trait and types
mod api;
pub use api::{MyModuleApi, MyModuleError, Input, Output};

// gRPC proto stubs
pub mod proto {
    tonic::include_proto!("my_module.v1");
}
pub use proto::my_module_service_client::MyModuleServiceClient;
pub use proto::my_module_service_server::{MyModuleService, MyModuleServiceServer};

// gRPC client
mod client;
pub use client::MyModuleGrpcClient;

// Wiring helpers
mod wiring;
pub use wiring::{wire_client, build_client};

/// Service name for discovery
pub const SERVICE_NAME: &str = "my_module.v1.MyModuleService";
```

### API Trait (in SDK)

```rust
// my_module-sdk/src/api.rs
use async_trait::async_trait;

/// API trait for MyModule
#[async_trait]
pub trait MyModuleApi: Send + Sync {
    async fn do_something(&self, input: Input) -> Result<Output, MyModuleError>;
}

/// Error type for MyModule operations
#[derive(thiserror::Error, Debug)]
pub enum MyModuleError {
    #[error("gRPC transport error: {0}")]
    Transport(String),
    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(Clone, Debug)]
pub struct Input {
    pub value: String
}

#[derive(Clone, Debug)]
pub struct Output {
    pub result: String
}
```

### gRPC Client (in SDK)

All clients MUST use `modkit_transport_grpc::client` utilities for consistent
timeouts, keepalive, retry logic, and tracing integration:

```rust
// my_module-sdk/src/client.rs
use anyhow::Result;
use async_trait::async_trait;
use tonic::transport::Channel;
use modkit_transport_grpc::client::{connect_with_retry, GrpcClientConfig};
use crate::api::{MyModuleApi, MyModuleError, Input, Output};
use crate::proto::{MyModuleServiceClient, DoSomethingRequest};

pub struct MyModuleGrpcClient {
    inner: MyModuleServiceClient<Channel>,
}

impl MyModuleGrpcClient {
    pub async fn connect(endpoint: &str) -> Result<Self> {
        let cfg = GrpcClientConfig::new("my_module");
        Self::connect_with_retry(endpoint, &cfg).await
    }

    pub async fn connect_with_retry(
        endpoint: impl Into<String>,
        cfg: &GrpcClientConfig,
    ) -> Result<Self> {
        let channel: Channel = connect_with_retry(endpoint, cfg).await?;
        Ok(Self { inner: MyModuleServiceClient::new(channel) })
    }
}

#[async_trait]
impl MyModuleApi for MyModuleGrpcClient {
    async fn do_something(&self, input: Input) -> Result<Output, MyModuleError> {
        let request = DoSomethingRequest { value: input.value };
        let response = self.inner.clone()
            .do_something(tonic::Request::new(request))
            .await
            .map_err(|e| MyModuleError::Transport(e.to_string()))?;
        Ok(Output { result: response.into_inner().result })
    }
}
```

### Wiring Helper (in SDK)

```rust
// my_module-sdk/src/wiring.rs
use std::sync::Arc;
use anyhow::Result;
use modkit::client_hub::ClientHub;
use module_orchestrator_contracts::DirectoryApi;
use crate::{MyModuleApi, MyModuleGrpcClient, SERVICE_NAME};

/// Wire the gRPC client into ClientHub
pub async fn wire_client(hub: &ClientHub, resolver: &dyn DirectoryApi) -> Result<()> {
    let endpoint = resolver.resolve_grpc_service(SERVICE_NAME).await?;
    let client = MyModuleGrpcClient::connect(&endpoint.uri).await?;
    hub.register::<dyn MyModuleApi>(Arc::new(client));
    tracing::info!(service = SERVICE_NAME, "client wired into ClientHub");
    Ok(())
}

/// Build client directly (without registering in hub)
pub async fn build_client(resolver: &dyn DirectoryApi) -> Result<Arc<dyn MyModuleApi>> {
    let endpoint = resolver.resolve_grpc_service(SERVICE_NAME).await?;
    let client = MyModuleGrpcClient::connect(&endpoint.uri).await?;
    Ok(Arc::new(client))
}
```

### Usage Example

```rust
// Consumer module
use my_module_sdk::{MyModuleApi, wire_client};

async fn init(&self, ctx: &ModuleCtx) -> Result<()> {
    let directory = ctx.client_hub().get::<dyn DirectoryApi>()?;

    // Wire the client into ClientHub
    wire_client(ctx.client_hub(), directory.as_ref()).await?;

    // Later, get client from ClientHub
    let client = ctx.client_hub().get::<dyn MyModuleApi>()?;
    let result = client.do_something(Input { value: "test".into() }).await?;
    Ok(())
}
```

### Example: calculator

See `examples/oop-modules/calculator/` for a complete working example:

* **calculator-sdk** — API trait, types, gRPC client, proto stubs, and `wire_client()` helper
* **calculator** — OoP module with gRPC server implementation

---

## REST with `OperationBuilder`

`OperationBuilder` is a type-state builder that **won't compile** unless you set both a **handler** and at least one *
*response** before calling `register()`. It also attaches request bodies and component schemas using `utoipa`.

### Quick reference

**Constructors**

```rust
OperationBuilder::<Missing, Missing, S>::get("/my-module/v1/path")
OperationBuilder::<Missing, Missing, S>::post("/my-module/v1/path")
// put/patch/delete are available too
```

**Describe**

```rust
.operation_id("module.op")
.summary("Short summary")
.description("Longer description")
.tag("group")
.path_param("id", "ID description")
.query_param("q", /*required=*/false, "Query description")
.query_param_typed("limit", false, "Max results", "integer")
```

**Request body (JSON)**

```rust
// Auto-register schema for T with utoipa::ToSchema; with/without description:
.json_request::<T>(openapi, "body description")
.json_request_no_desc::<T>(openapi)

// Or use pre-registered schema by name:
.json_request_schema("SchemaName", "body description")
.json_request_schema_no_desc("SchemaName")

// Make request body optional (default is required):
.request_optional()
```

**Request body (file uploads)**

```rust
// Multipart form file upload (single file field):
.multipart_file_request("file", Some("File to upload"))

// Raw binary body (application/octet-stream):
.octet_stream_request(Some("Raw file bytes"))
```

**MIME type validation**

```rust
// Configure allowed Content-Type values (enforced by ingress middleware):
.allow_content_types( & ["application/json", "application/xml"])
```

**Responses**

```rust
// First response (Missing -> Present):
.json_response(StatusCode::OK, "OK")
.text_response(StatusCode::OK, "OK", "text/plain")
.html_response(StatusCode::OK, "HTML")

// Schema-aware JSON responses (auto-register T):
.json_response_with_schema::<T>(openapi, StatusCode::OK, "Success")

// RFC-9457 problem responses:
.problem_response(openapi, StatusCode::BAD_REQUEST, "Bad request")
.problem_response(openapi, StatusCode::CONFLICT, "Conflict")
.problem_response(openapi, StatusCode::INTERNAL_SERVER_ERROR, "Internal error")

// Server-Sent Events (SSE) responses:
.sse_json::<T>(openapi, "Real-time event stream")

// Add all standard error responses (400, 401, 403, 404, 409, 422, 429, 500):
.standard_errors(openapi)

// Add 422 validation error with structured ValidationError schema:
.with_422_validation_error(openapi)
```

**Authentication**

```rust
// Require authentication with resource:action permission (module-defined Resource/Action enums):
.require_auth(&Resource::Users, &Action::Read)

// Or mark as public (no auth required):
.public()
```

**Licensing (feature gating)**

```rust
use modkit::api::operation_builder::LicenseFeature;

// Define a license feature identifier for this module.
// AsRef<str> should return the canonical feature id (typically a GTS type name).
struct BaseLicenseFeature;

impl AsRef<str> for BaseLicenseFeature {
    fn as_ref(&self) -> &'static str {
        "gts.x.core.lic.feat.v1~x.core.global.base.v1"
    }
}

impl LicenseFeature for BaseLicenseFeature {}

// For authenticated endpoints, calling `require_license_features(...)` is mandatory.
// Use an empty iterator (e.g. `[]`) to explicitly declare that no license feature is required.
.require_auth(&Resource::Users, &Action::Read)
.require_license_features::<BaseLicenseFeature>([])

// Or require one (or more) features for this operation:
let feature = BaseLicenseFeature;
.require_auth(&Resource::Users, &Action::Read)
.require_license_features([&feature])
```

Notes:

- Authenticated operations must call `require_license_features(...)` before `register(...)`.
- Public routes cannot (and do not need to) call `require_license_features(...)`.
- `api_ingress` currently enforces license requirements via a stub middleware that only allows the base feature.

**Handler / method router**

```rust
.handler(my_function_handler)    // preferred: free functions using State<S>
.method_router(my_method_router) // advanced: per-route middleware/layers
```

**Register**

```rust
.register(router, openapi) -> Router<S>
```

### Using Router state (`S`)

Pass a state once via `Router::with_state(S)`. Handlers are free functions taking `State<S>`, so you don't capture/clone
your service per route.

---

## Error handling (RFC-9457)

ModKit provides centralized types in `modkit::api::problem`:

* `Problem` — RFC-9457 Problem Details (implements `IntoResponse` directly)
* `ValidationError` — itemized validation error

**Handler example**

```rust
use modkit::api::problem::{Problem, bad_request, conflict, internal_error};
use axum::{extract::State, Json};
use http::StatusCode;

async fn create_user_handler(
    State(state): State<ApiState>,
    Json(req): Json<CreateUserReq>
) -> Result<(StatusCode, Json<UserDto>), Problem> {
    if req.email.is_empty() {
        return Err(bad_request("Email is required"));
    }

    match state.svc.create_user(req).await {
        Ok(user) => Ok((StatusCode::CREATED, Json(user.into()))),
        Err(DomainError::EmailAlreadyExists { email }) => {
            Err(conflict(format!("User with email '{}' already exists", email)))
        }
        Err(e) => {
            tracing::error!("Failed to create user: {}", e);
            Err(internal_error("User creation failed"))
        }
    }
}
```

**OpenAPI response registration**

```rust
OperationBuilder::post("/user-management/v1/users")
    .operation_id("users.create")
    .summary("Create user")
    .json_request::<CreateUserReq>(openapi, "User creation data")
    .handler(create_user_handler)
    .json_response_with_schema::<UserDto>(openapi, StatusCode::CREATED, "User created")
    .standard_errors(openapi)  // Adds 400, 401, 403, 404, 409, 422, 429, 500
    .register(router, openapi);

// Or for more specific error responses:
OperationBuilder::post("/user-management/v1/users")
    .operation_id("users.create")
    .summary("Create user")
    .json_request::<CreateUserReq>(openapi, "User creation data")
    .handler(create_user_handler)
    .json_response_with_schema::<UserDto>(openapi, StatusCode::CREATED, "User created")
    .problem_response(openapi, StatusCode::BAD_REQUEST, "Invalid input")
    .problem_response(openapi, StatusCode::CONFLICT, "Email already exists")
    .with_422_validation_error(openapi)  // Structured validation errors
    .problem_response(openapi, StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
    .register(router, openapi);
```

---

# Modkit Unified Pagination/OData System

## Layers

- `modkit-odata`: AST, ODataQuery, CursorV1, ODataOrderBy, SortDir, ODataPageError, **Page<T>/PageInfo**.
- `modkit`: HTTP extractor for OData (`$filter`, `$orderby`, `limit`, `cursor`) with budgets + Problem mapper.
- `modkit-db`: Type-safe OData filter system with `FilterField` trait, `FilterNode<F>` AST, and SeaORM integration.

## Architecture (Type-Safe OData)

The OData system uses a **three-layer architecture** for type safety:

### 1. DTO Layer (REST)

Use `#[derive(ODataFilterable)]` on your REST DTOs to auto-generate a `FilterField` enum:

```rust
use modkit_db_macros::ODataFilterable;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, ODataFilterable)]
pub struct UserDto {
    #[odata(filter(kind = "Uuid"))]
    pub id: Uuid,
    #[odata(filter(kind = "String"))]
    pub email: String,
    #[odata(filter(kind = "DateTimeUtc"))]
    pub created_at: DateTime<Utc>,
    pub display_name: String,  // no #[odata] = not filterable
}
```

This generates a `UserDtoFilterField` enum automatically with variants for each filterable field.

**Supported field kinds**: `String`, `I64`, `F64`, `Bool`, `Uuid`, `DateTimeUtc`, `Date`, `Time`, `Decimal`

### 2. Domain/Service Layer

Work with transport-agnostic `FilterNode<F>` AST - no HTTP or SeaORM dependencies:

```rust
use modkit_db::odata::filter::FilterNode;
use crate::api::rest::dto::UserDtoFilterField;

pub struct UserService {
    /* ... */
}

impl UserService {
    pub async fn list_users(
        &self,
        filter: Option<FilterNode<UserDtoFilterField>>,
        order: ODataOrderBy,
        limit: u64,
    ) -> Result<Page<User>, DomainError> {
        self.repo.list_with_odata(filter, order, limit).await
    }
}
```

### 3. Infrastructure Layer

Map FilterField to SeaORM columns via `ODataFieldMapping` trait:

```rust
use modkit_db::odata::sea_orm_filter::{FieldToColumn, ODataFieldMapping};

pub struct UserODataMapper;

impl FieldToColumn<UserDtoFilterField> for UserODataMapper {
    type Column = Column;  // SeaORM Column enum

    fn map_field(field: UserDtoFilterField) -> Column {
        match field {
            UserDtoFilterField::Id => Column::Id,
            UserDtoFilterField::Email => Column::Email,
            UserDtoFilterField::CreatedAt => Column::CreatedAt,
        }
    }
}

impl ODataFieldMapping<UserDtoFilterField> for UserODataMapper {
    type Entity = Entity;

    fn extract_cursor_value(model: &Model, field: UserDtoFilterField) -> sea_orm::Value {
        match field {
            UserDtoFilterField::Id => sea_orm::Value::Uuid(Some(Box::new(model.id))),
            UserDtoFilterField::Email => sea_orm::Value::String(Some(Box::new(model.email.clone()))),
            UserDtoFilterField::CreatedAt => sea_orm::Value::ChronoDateTimeUtc(Some(Box::new(model.created_at))),
        }
    }
}
```

### Repository Usage

```rust
use modkit_db::odata::sea_orm_filter::{paginate_odata, LimitCfg};

pub async fn list_with_odata(
    &self,
    filter: Option<FilterNode<UserDtoFilterField>>,
    order: ODataOrderBy,
    limit: u64,
) -> Result<Page<User>, RepoError> {
    let odata_query = ODataQuery {
        filter: filter.map(|f| /* convert to AST string if needed */),
        order: Some(order),
        limit: Some(limit),
        cursor: None,
        filter_hash: None,
    };

    let page = paginate_odata::<UserDtoFilterField, UserODataMapper, _, _, _, _>(
        base_query,
        conn,
        &odata_query,
        ("id", SortDir::Desc),  // tiebreaker
        LimitCfg { default: 25, max: 1000 },
        |model| model.into(),  // map to domain
    ).await?;

    Ok(page)
}
```

## Usage (4 steps)

1. In REST DTO: `#[derive(ODataFilterable)]` with `#[odata(filter(kind = "..."))]` on filterable fields.
2. In the handler: `OData(q)` extractor (Axum) → pass `q` down to service.
3. In repo/infra: implement `ODataFieldMapping<F>` mapper, call `paginate_odata(...)` and return `Page<T>`.
4. In REST: map `ODataError` to Problem via `odata_page_error_to_problem`.

### Notes

- If `cursor` present, `$orderby` must be omitted (400 ORDER_WITH_CURSOR).
- Cursors are opaque, Base64URL v1; include signed order `s` and filter hash `f`.
- Order must include a unique tiebreaker (e.g., `id`), enforced via helper.
- The `#[odata(filter(kind = "..."))]` attribute is required for each filterable field.
- Non-annotated fields are automatically excluded from filtering.

---

## Server-Sent Events (SSE)

ModKit provides built-in support for Server-Sent Events through the `SseBroadcaster<T>` type and `OperationBuilder`
integration. This enables real-time streaming of typed events to web clients with proper OpenAPI documentation.

### Core components

* **`SseBroadcaster<T>`** — Type-safe broadcaster built on `tokio::sync::broadcast`
* **`OperationBuilder::sse_json<T>()`** — Register SSE endpoints with OpenAPI schemas
* **Domain events** — Transport-agnostic events published by the domain layer
* **SSE adapters** — Bridge domain events to SSE transport

### Basic SSE broadcaster

```rust
use modkit::SseBroadcaster;
use serde::{Serialize, Deserialize};
use utoipa::ToSchema;

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct UserEvent {
    pub kind: String,
    pub id: uuid::Uuid,
    pub at: chrono::DateTime<chrono::Utc>,
}

// Create broadcaster with buffer capacity
let broadcaster = SseBroadcaster::<UserEvent>::new(1024);

// Send events
broadcaster.send(UserEvent {
kind: "created".to_string(),
id: uuid::Uuid::new_v4(),
at: chrono::Utc::now(),
});

// Subscribe to stream
let mut stream = broadcaster.subscribe_stream();
// Use stream.next().await to receive events
```

### SSE handler example

```rust
use axum::{extract::Extension, response::sse::Sse};
use futures::Stream;
use std::convert::Infallible;

async fn user_events_handler(
    Extension(sse): Extension<SseBroadcaster<UserEvent>>,
) -> Sse<impl Stream<Item=Result<axum::response::sse::Event, Infallible>>> {
    tracing::info!("New SSE connection for user events");
    sse.sse_response()  // Returns Sse with keepalive pings
}
```

### Register SSE routes

```rust
use axum::{Extension, Router};
use tower_http::timeout::TimeoutLayer;
use std::time::Duration;

fn register_sse_route(
    router: Router<S>,
    openapi: &dyn OpenApiRegistry,
    broadcaster: SseBroadcaster<UserEvent>,
) -> Router<S> {
    OperationBuilder::<Missing, Missing, S>::get("/user-management/v1/users/events")
        .operation_id("users.events")
        .summary("User events stream")
        .description("Real-time stream of user events via Server-Sent Events")
        .tag("users")
        .handler(user_events_handler)
        .sse_json::<UserEvent>(openapi, "SSE stream of UserEvent")
        .register(router, openapi)
        .layer(Extension(broadcaster))
        .layer(TimeoutLayer::new(Duration::from_secs(3600))) // 1 hour timeout
}
```

### Domain-driven SSE architecture

For clean separation of concerns, use domain events with adapter pattern:

**1. Domain events (transport-agnostic)**

```rust
#[derive(Debug, Clone)]
pub enum UserDomainEvent {
    Created { id: Uuid, at: DateTime<Utc> },
    Updated { id: Uuid, at: DateTime<Utc> },
    Deleted { id: Uuid, at: DateTime<Utc> },
}
```

**2. Domain port (output interface)**

```rust
pub trait EventPublisher<E>: Send + Sync + 'static {
    fn publish(&self, event: &E);
}
```

**3. Domain service (publishes events)**

```rust
use std::sync::Arc;

pub struct UserService {
    repo: Arc<dyn UsersRepository>,
    events: Arc<dyn EventPublisher<UserDomainEvent>>,
}

impl UserService {
    pub async fn create_user(&self, data: NewUser) -> Result<User, DomainError> {
        let user = self.repo.create(data).await?;

        // Publish domain event
        self.events.publish(&UserDomainEvent::Created {
            id: user.id,
            at: user.created_at,
        });

        Ok(user)
    }
}
```

**4. SSE adapter (implements domain port)**

```rust
use modkit::SseBroadcaster;

pub struct SseUserEventPublisher {
    broadcaster: SseBroadcaster<UserEvent>,
}

impl EventPublisher<UserDomainEvent> for SseUserEventPublisher {
    fn publish(&self, event: &UserDomainEvent) {
        let sse_event = UserEvent::from(event);  // Convert domain -> transport
        self.broadcaster.send(sse_event);
    }
}

impl From<&UserDomainEvent> for UserEvent {
    fn from(e: &UserDomainEvent) -> Self {
        use UserDomainEvent::*;
        match e {
            Created { id, at } => Self { kind: "created".into(), id: *id, at: *at },
            Updated { id, at } => Self { kind: "updated".into(), id: *id, at: *at },
            Deleted { id, at } => Self { kind: "deleted".into(), id: *id, at: *at },
        }
    }
}
```

**5. Module wiring**

```rust
#[modkit::module(name = "users", capabilities = [db, rest])]
pub struct UsersModule {
    service: ArcSwapOption<UserService>,
    sse_broadcaster: SseBroadcaster<UserEvent>,
}

impl Module for UsersModule {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        let repo = Arc::new(SqlUsersRepository::new(ctx.db.clone()));

        // Create SSE adapter that implements domain port
        let event_publisher: Arc<dyn EventPublisher<UserDomainEvent>> =
            Arc::new(SseUserEventPublisher::new(self.sse_broadcaster.clone()));

        let service = UserService::new(repo, event_publisher);
        self.service.store(Some(Arc::new(service)));
        Ok(())
    }
}

impl RestfulModule for UsersModule {
    fn register_rest(&self, _ctx: &ModuleCtx, router: Router, openapi: &dyn OpenApiRegistry) -> anyhow::Result<Router> {
        let router = register_crud_routes(router, openapi, self.service.clone())?;
        let router = register_sse_route(router, openapi, self.sse_broadcaster.clone());
        Ok(router)
    }
}
```

### SSE response variants

The `SseBroadcaster` provides several response methods:

```rust
// Basic SSE with keepalive pings
broadcaster.sse_response()

// SSE with custom HTTP headers
broadcaster.sse_response_with_headers([
(HeaderName::from_static("x-custom"), HeaderValue::from_static("value"))
])

// Named events (sets event: field in SSE stream)
broadcaster.sse_response_named("user-events")

// Named events with custom headers
broadcaster.sse_response_named_with_headers("user-events", headers)
```

### OpenAPI integration

SSE endpoints are automatically documented as `text/event-stream` responses with proper schema references:

```yaml
paths:
  /users/events:
    get:
      summary: User events stream
      responses:
        '200':
          description: SSE stream of UserEvent
          content:
            text/event-stream:
              schema:
                $ref: '#/components/schemas/UserEvent'
```

### Best practices

* Use **bounded channels** (e.g., 1024 capacity) to prevent memory leaks from slow clients
* Apply **timeout middleware** for long-lived SSE connections (e.g., 1-hour timeout)
* Keep **domain events transport-agnostic** - use adapter pattern for SSE integration
* **Inject broadcasters per-route** via `Extension` rather than global state
* Use **structured event types** with `kind` field for client-side filtering
* Include **timestamps** for event ordering and debugging

---

## File upload endpoints

ModKit provides convenient helpers for file upload endpoints with proper OpenAPI documentation.

### Multipart form file upload

For traditional HTML form uploads with a single file field:

```rust
use axum::{extract::Multipart, Extension};
use modkit::api::problem::{Problem, internal_error};

async fn upload_handler(
    Extension(service): Extension<Arc<MyService>>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, Problem> {
    // Extract file from multipart form
    while let Some(field) = multipart.next_field().await.map_err(|e| internal_error(e))? {
        if field.name() == Some("file") {
            let filename = field.file_name().map(|s| s.to_string());
            let bytes = field.bytes().await.map_err(|e| internal_error(e))?;

            let result = service.process_file(filename, bytes).await?;
            return Ok(Json(result));
        }
    }
    Err(bad_request("Missing 'file' field in multipart form"))
}

// Register with type-safe builder
OperationBuilder::post("/files/v1/upload")
    .operation_id("files.upload")
    .summary("Upload a file")
    .multipart_file_request("file", Some("File to upload"))
    .handler(upload_handler)
    .json_response_with_schema::<UploadResponse>(openapi, StatusCode::OK, "Upload successful")
    .standard_errors(openapi)
    .register(router, openapi);
```

The `.multipart_file_request()` method:

- Sets `multipart/form-data` content type
- Generates proper OpenAPI schema with binary file field
- Restricts allowed Content-Type to `multipart/form-data` only
- Produces UI-friendly documentation for tools like Stoplight Elements

### Raw binary upload (octet-stream)

For endpoints that accept the entire request body as raw bytes:

```rust
use axum::{body::Bytes, Extension};

async fn upload_binary_handler(
    Extension(service): Extension<Arc<MyService>>,
    body: Bytes,
) -> Result<Json<ParseResponse>, Problem> {
    let result = service.parse_bytes(body).await?;
    Ok(Json(result))
}

// Register with type-safe builder
OperationBuilder::post("/files/v1/upload")
    .operation_id("files.upload_binary")
    .summary("Upload raw file bytes")
    .octet_stream_request(Some("Raw file bytes"))
    .handler(upload_binary_handler)
    .json_response_with_schema::<ParseResponse>(openapi, StatusCode::OK, "Parse successful")
    .standard_errors(openapi)
    .register(router, openapi);
```

The `.octet_stream_request()` method:

- Sets `application/octet-stream` content type
- Generates OpenAPI schema: `type: string, format: binary`
- Restricts allowed Content-Type to `application/octet-stream` only
- Tools render this as a single file upload control for the entire body

### MIME type validation

Both helpers automatically configure MIME type validation via the ingress middleware. If a request arrives with a
different Content-Type, it will receive HTTP 415 (Unsupported Media Type).

You can also manually configure allowed types:

```rust
OperationBuilder::post("/files/v1/upload")
    .operation_id("files.upload_custom")
    .summary("Upload with custom validation")
    .allow_content_types(&["application/pdf", "image/png", "image/jpeg"])
    .handler(upload_handler)
    .json_response(StatusCode::OK, "Upload successful")
    .problem_response(openapi, StatusCode::UNSUPPORTED_MEDIA_TYPE, "Unsupported file type")
    .register(router, openapi);
```

**Important**: `.allow_content_types()` is independent of the request body schema. It only configures ingress validation
and doesn't create OpenAPI request body specs. Use it when you want to enforce MIME types but handle the body parsing
manually in your handler.

---

## Idiomatic conversions

Prefer `From` over ad-hoc mapper functions.

```rust
// Convert DB entity to contract model (by value)
impl From<UserEntity> for User {
    fn from(e: UserEntity) -> Self {
        Self {
            id: e.id,
            email: e.email,
            display_name: e.display_name,
            created_at: e.created_at,
            updated_at: e.updated_at,
        }
    }
}

// Convert by reference (avoids moving the entity)
impl From<&UserEntity> for User {
    fn from(e: &UserEntity) -> Self {
        Self {
            id: e.id,
            email: e.email.clone(),
            display_name: e.display_name.clone(),
            created_at: e.created_at,
            updated_at: e.updated_at,
        }
    }
}

// Usage
let user: User = entity.into();
let users: Vec<User> = entities.into_iter().map(Into::into).collect();
```

---

## OpenAPI integration (utoipa)

ModKit provides a standalone OpenAPI registry system that collects operation specs and schemas, then builds a complete
OpenAPI 3.1 document.

### OpenApiRegistry trait

The `OpenApiRegistry` trait provides:

* `register_operation(&self, spec: &OperationSpec)` - Register API operations
* `ensure_schema_raw(&self, name: &str, schemas: SchemaCollection) -> String` - Register component schemas
* Helper function `ensure_schema::<T>()` - Type-safe schema registration with transitive dependencies

### Implementation

The `OpenApiRegistryImpl` uses lock-free data structures for high performance:

```rust
use modkit::api::{OpenApiRegistry, OpenApiRegistryImpl, OpenApiInfo};

// Create registry
let registry = OpenApiRegistryImpl::new();

// Register operations (done automatically by OperationBuilder)
// ...

// Build OpenAPI document
let info = OpenApiInfo {
title: "My API".to_string(),
version: "1.0.0".to_string(),
description: Some("API documentation".to_string()),
};
let openapi = registry.build_openapi( & info) ?;

// Serialize to JSON
let json = serde_json::to_string_pretty( & openapi) ?;
```

### Schema registration

DTOs derive `utoipa::ToSchema`. The `OperationBuilder` methods automatically register schemas when you use:

* `.json_request::<T>()` - Registers request body schema
* `.json_response_with_schema::<T>()` - Registers response schema
* `.sse_json::<T>()` - Registers SSE event schema
* `.problem_response()` - Registers Problem schema
* `.with_422_validation_error()` - Registers ValidationError schema

**DTO example**

```rust
use serde::{Serialize, Deserialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
#[schema(title = "UserDto", description = "User representation for REST")]
pub struct UserDto {
    pub id: uuid::Uuid,
    pub email: String,
    pub display_name: String,
    #[schema(format = "date-time")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[schema(format = "date-time")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
```

### Security schemes

The registry automatically adds a `bearerAuth` security scheme (HTTP Bearer with JWT format). Operations using
`.require_auth()` will reference this scheme in OpenAPI.

### Content type handling

* `application/json` and `application/problem+json` reference component schemas
* `text/event-stream` (SSE) references event schemas
* `multipart/form-data` generates object schemas with binary file fields
* `application/octet-stream` generates string schemas with binary format
* Other content types use string schemas with custom formats

---

## Typed ClientHub

The **ClientHub** provides type-safe client resolution for inter-module communication. It supports both in-process and
remote clients:

* **In-process clients** — direct function calls within the same process
* **Remote clients** — gRPC clients for OoP modules (resolved via DirectoryApi)
* **Scoped clients** — multiple implementations of the same interface keyed by scope (for plugins)

**Client types:**

* **`contract::client`** defines the trait & DTOs exposed to other modules.
* **`gateways/local.rs`** implements that trait for in-process communication.
* **`*-grpc/src/client.rs`** implements the trait for remote gRPC communication.
* Consumers resolve the typed client from ClientHub by interface type (+ optional scope).

### In-Process vs Remote Clients

| Aspect       | In-Process           | Remote (OoP)                  |
|--------------|----------------------|-------------------------------|
| Transport    | Direct call          | gRPC                          |
| Latency      | Nanoseconds          | Milliseconds                  |
| Isolation    | Shared process       | Separate process              |
| Contract     | Trait in `contract/` | Trait in `*-contracts/` crate |
| Registration | `expose_*_client()`  | DirectoryApi + gRPC client    |

**Publish in `init`**

```rust
#[async_trait::async_trait]
impl Module for MyModule {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        let cfg = ctx.module_config::<crate::config::Config>();
        let svc = std::sync::Arc::new(domain::service::MyService::new(ctx.db.clone(), cfg));
        self.service.store(Some(svc.clone()));

        let api: std::sync::Arc<dyn contract::client::MyModuleApi> =
            std::sync::Arc::new(gateways::local::MyModuleLocalClient::new(svc));

        expose_my_module_client(ctx, &api)?;
        Ok(())
    }
}
```

**Consume**

```rust
let api = my_module_client( & ctx.client_hub);
// or:
let api = ctx.client_hub.get::<dyn my_module::contract::client::MyModuleApi>() ?;
```

### Scoped Clients (for Plugins)

For plugin-like scenarios where multiple implementations of the same interface coexist, use scoped clients:

```rust
use modkit::client_hub::ClientScope;

// Plugin registers with a scope (e.g., GTS instance ID)
let scope = ClientScope::gts_id("gts.x.core.modkit.plugin.v1~vendor.pkg.my_module.plugin.v1~acme.test._.plugin.v1");
ctx.client_hub().register_scoped::<dyn MyPluginApi>(scope, plugin_impl);

// Gateway resolves the selected plugin
let scope = ClientScope::gts_id(&selected_instance_id);
let plugin = ctx.client_hub().get_scoped::<dyn MyPluginApi>(&scope)?;
```

**Key points:**

* Scoped clients are independent from global (unscoped) clients
* Use `ClientScope::gts_id()` for GTS-based plugin IDs
* See [MODKIT_PLUGINS.md](./MODKIT_PLUGINS.md) for the complete plugin architecture guide

---

## Contracts & lifecycle traits

```rust
#[async_trait::async_trait]
pub trait Module: Send + Sync + 'static {
    async fn init(&self, ctx: &crate::context::ModuleCtx) -> anyhow::Result<()>;
}

#[async_trait::async_trait]
pub trait DbModule: Send + Sync {
    async fn migrate(&self, db: &db::DbHandle) -> anyhow::Result<()>;
}

pub trait RestfulModule: Send + Sync {
    fn register_rest(
        &self,
        ctx: &crate::context::ModuleCtx,
        router: axum::Router,
        openapi: &dyn crate::api::OpenApiRegistry,
    ) -> anyhow::Result<axum::Router>;
}

#[async_trait::async_trait]
pub trait StatefulModule: Send + Sync {
    async fn start(&self, cancel: tokio_util::sync::CancellationToken) -> anyhow::Result<()>;
    async fn stop(&self, cancel: tokio_util::sync::CancellationToken) -> anyhow::Result<()>;
}
```

**Order:** `init → migrate → register_rest → start → stop` (topologically sorted by `deps`).

---

## Testing

* **Unit test** domain services by mocking infra.
* **REST test** handlers with `Router::oneshot` and a stub `ApiState`.
* **Integration test** module wiring: call `init`, resolve typed clients from ClientHub, assert behavior.
* For stateful modules, exercise lifecycle: start with a `CancellationToken`, signal shutdown, assert transitions.

---

## Addendum — Rationale (DDD-light)

1. **What does a domain service do?**
   Encodes **business rules/orchestration**. It calls repositories/infrastructure, applies invariants, aggregates data,
   owns retries/timeouts at the business level.

2. **Where to put “low-level” things?**
   In **infra/** (storage, system probes, processes, files, raw SQL, HTTP to other systems). Domain calls infra via
   small interfaces/constructors.

3. **Where to keep “glue”?**
   Glue that adapts domain to transport lives in **api/rest** (HTTP DTOs, handlers). Glue that adapts domain to **other
   modules** lives in **gateways/** (client implementations). DB mapping glue sits in **infra/storage**.

4. **Why not put platform-dependent logic into service?**
   To keep business rules portable/testable. Platform logic churns often; isolating it in infra avoids leaking OS/DB
   concerns into your domain.

5. **What is `contract` and why separate?**
   It’s the **public API** of your module for **other modules**: traits + DTOs + domain errors safe to expose. This
   separation allows swapping local/remote clients without changing consumers. For simple internal modules you may
   re-export a subset of domain models via `contract::model`.

6. **How to hide domain & internals from other modules?**
   Re-export only what’s needed via `contract`. Consumers depend on `contract` and `gateways` through the ClientHub;
   they never import your domain/infra directly.

---

## OoP Configuration: How It Works

This section describes the complete configuration flow for Out-of-Process modules, from master host preparation to final
merged configuration in the OoP process.

### Configuration Flow Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           MASTER HOST                                   │
├─────────────────────────────────────────────────────────────────────────┤
│  quickstart.yaml                                                        │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │ database:                                                        │   │
│  │   servers:                                                       │   │
│  │     sqlite_main: { params: { WAL: true }, pool: { max_conns: 5 }}│   │
│  │                                                                  │   │
│  │ modules:                                                         │   │
│  │   calculator:                                                  │   │
│  │     runtime: { type: oop, execution: { ... } }                   │   │
│  │     database: { server: sqlite_main, file: accum.db }            │   │
│  │     config: { setting: "master_value" }                          │   │
│  │                                                                  │   │
│  │ logging:                                                         │   │
│  │   calculator: { console_level: info, file: logs/accum.log }    │   │
│  │                                                                  │   │
│  │ tracing: { enabled: true, ... }                                  │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                              │                                          │
│                              ▼                                          │
│               render_module_config_for_oop()                            │
│                              │                                          │
│                              ▼                                          │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │ RenderedModuleConfig (JSON):                                     │   │
│  │   database: { global: {...}, module: {...} }  ← structured       │   │
│  │   config: { setting: "master_value" }                            │   │
│  │   logging: { calculator: {...}, default: {...} }               │   │
│  │   tracing: { enabled: true, ... }                                │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                              │                                          │
│                              ▼                                          │
│         MODKIT_MODULE_CONFIG env var + spawn OoP process                │
└─────────────────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                           OoP PROCESS                                   │
├─────────────────────────────────────────────────────────────────────────┤
│  1. Load local --config file (oop-example.yaml)                         │
│  2. Parse MODKIT_MODULE_CONFIG env var                                  │
│  3. Merge configs: master (base) + local (override)                     │
│                                                                         │
│  ┌──────────────────────┐    ┌──────────────────────┐                   │
│  │ Master (from env)    │ +  │ Local (from --config)│                   │
│  │ database: {...}      │    │ database:            │                   │
│  │ config: {...}        │    │   pool: { max: 10 }  │                   │
│  │ logging: {...}       │    │ config: { x: "y" }   │                   │
│  │ tracing: {...}       │    │ logging: {...}       │                   │
│  └──────────────────────┘    └──────────────────────┘                   │
│                │                       │                                │
│                └───────────┬───────────┘                                │
│                            ▼                                            │
│                   build_oop_config_and_db()                             │
│                            │                                            │
│                            ▼                                            │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │ Final merged configuration:                                      │   │
│  │   database: field-by-field merge → DbManager                     │   │
│  │   config: local replaces master (if present)                     │   │
│  │   logging: key-by-key merge → init_logging_unified()             │   │
│  │   tracing: from master only → OTEL layer                         │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                            │                                            │
│                            ▼                                            │
│                 Module lifecycle starts                                 │
└─────────────────────────────────────────────────────────────────────────┘
```

### Bootstrap Sequence in OoP Process

When an OoP module starts via `run_oop_with_options()`:

1. **Load local config file** — `AppConfig::load_or_default(opts.config_path)`
2. **Parse rendered config from master** — `MODKIT_MODULE_CONFIG` env var → `RenderedModuleConfig`
3. **Merge configurations** — `build_oop_config_and_db()` merges all sections
4. **Initialize logging** — Uses merged logging config (master + local)
5. **Initialize OTEL tracing** — Uses master's tracing config
6. **Create DbManager** — Uses merged database config via Figment
7. **Connect to DirectoryService** — Uses `MODKIT_DIRECTORY_ENDPOINT`
8. **Start module lifecycle** — Normal `init → migrate → start` flow

### Environment Variables

OoP modules receive configuration from the master host via environment variables:

| Variable                    | Description                                                                |
|-----------------------------|----------------------------------------------------------------------------|
| `MODKIT_MODULE_CONFIG`      | JSON blob with rendered module config (database, config, logging, tracing) |
| `MODKIT_DIRECTORY_ENDPOINT` | DirectoryService gRPC endpoint (e.g., `http://127.0.0.1:50051`)            |
| `MODKIT_CONFIG_PATH`        | Path to config file (fallback if `MODKIT_MODULE_CONFIG` not set)           |

### Merge Strategies by Section

For each configuration section, master config serves as **base** and local `--config` serves as **override**:

| Section      | Merge Strategy       | Description                                                                                 |
|--------------|----------------------|---------------------------------------------------------------------------------------------|
| **database** | Field-by-field merge | Local fields override master fields (dsn, host, port, user, password, dbname, params, pool) |
| **logging**  | Key-by-key merge     | Local subsystem keys override master keys (e.g., "default", "calculator")                   |
| **config**   | Full replacement     | If local has `config` section, it completely replaces master                                |
| **tracing**  | From master only     | OoP modules use master's OTEL settings                                                      |

**Database configuration merge (3 levels):**

```
1. database.servers.*           (global templates in master)
        ↓ merge
2. modules.<name>.database      (module section in master)
        ↓ merge
3. modules.<name>.database      (module section in local --config)
        ↓
   Final database configuration
```

Each level uses field-by-field merge (same logic as `DbManager::merge_server_into_module`).

**Example:**

Master config (`quickstart.yaml`):

```yaml
database:
  servers:
    sqlite_main:
      params:
        WAL: "true"
      pool:
        max_conns: 5

modules:
  calculator:
    database:
      server: "sqlite_main"
      file: "accum.db"
    config:
      some_setting: "from_master"

logging:
  calculator:
    console_level: info
    file: "logs/calculator.log"
```

Local OoP config (`oop-example.yaml`):

```yaml
modules:
  calculator:
    database:
      pool:
        max_conns: 10  # Overrides master's 5
    config:
      some_setting: "from_local"  # Replaces master entirely

logging:
  calculator:
    console_level: debug  # Overrides master's info
    file: "logs/calculator-oop.log"  # Overrides master's file
```

**Result:**

- Database: Uses `sqlite_main` server with `max_conns: 10` (local override)
- Config: `some_setting: "from_local"` (local replaces master)
- Logging: `console_level: debug`, `file: "logs/calculator-oop.log"` (local overrides)

### Standalone Mode

When `MODKIT_MODULE_CONFIG` is not set (e.g., running OoP module manually for debugging):

- OoP module uses **only** local `--config` file
- No merge happens — local config is the complete configuration
- Useful for local development and testing

```bash
# Standalone mode - no master host
./calculator-oop --config config/oop-example.yaml
```

### Log Forwarding from OoP to Master

When master host spawns an OoP module:

1. Master captures OoP's stdout/stderr
2. `detect_log_level()` parses log level from each line (supports both plain text and JSON formats)
3. Logs are re-emitted via master's tracing with `oop_module` and `oop_instance_id` context

**Supported log formats:**

```
# Plain text (tracing-subscriber default)
2025-12-09T10:00:00Z  INFO calculator: Starting...
2025-12-09T10:00:00Z DEBUG calculator: Processing...

# JSON format (tracing-subscriber json layer)
{"timestamp":"2025-12-09T10:00:00Z","level":"INFO","message":"Starting..."}
{"timestamp":"2025-12-09T10:00:00Z","level":"DEBUG","message":"Processing..."}
```

Both formats are detected and forwarded with the correct log level.

---

## Best practices

* Handlers are thin; domain services are cohesive and testable.
* Keep DTO mapping in `api/rest/dto.rs`; don't leak HTTP types into domain.
* Prefer `ArcSwap`/lock-free caches for read-mostly state.
* Use `tracing` with module/operation fields.
* Keep migrations in `infra/storage/migrations/` and run them in `DbModule::migrate`.
* For SSE: use bounded channels, domain events with adapters, and per-route injection.
* Use `.standard_errors(openapi)` for consistent error responses across your API.
* Use `.multipart_file_request()` for HTML form uploads, `.octet_stream_request()` for raw binary.
* Use `.require_auth()` for protected endpoints, `.public()` for public endpoints.
* Use `.with_422_validation_error()` for endpoints with input validation.
* For OoP modules: use the SDK pattern with a single `*-sdk` crate containing API trait, types, gRPC client, and wiring
  helpers.
* For gRPC: server implementations live in the module itself; the SDK crate provides only the client.
* For gRPC clients: always use `modkit_transport_grpc::client` utilities (`connect_with_stack`, `connect_with_retry`).
* Use `CancellationToken` for coordinated shutdown across the entire process tree.
* For plugin systems: use scoped ClientHub registration and GTS-based discovery (see [MODKIT_PLUGINS.md](./MODKIT_PLUGINS.md)).
  - **Gateway** registers the plugin **schema** (GTS type definition)
  - **Plugins** register their **instances** (metadata + scoped client)
* For GTS schema generation: use `gts_schema_with_refs_as_string()`. This method is faster (static), automatically sets
  the correct `$id`, and generates proper `$ref` references.
