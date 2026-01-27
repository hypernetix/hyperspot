# Out-of-Process Modules and gRPC SDK Pattern

ModKit supports running modules as separate processes with gRPC-based inter-process communication. This enables process isolation, language flexibility, and independent scaling.

## Core invariants

- **Rule**: For OoP modules, use the SDK pattern with a single `*-sdk` crate containing API trait, types, gRPC client, and wiring helpers.
- **Rule**: For gRPC: server implementations live in the module itself; the SDK crate provides only the client.
- **Rule**: For gRPC clients: always use `modkit_transport_grpc::client` utilities (`connect_with_stack`, `connect_with_retry`).
- **Rule**: Use `CancellationToken` for coordinated shutdown across the entire process tree.

## RuntimeKind

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

## OoP Module Configuration

### YAML configuration

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

### Configuration fields

- `type: oop` — marks the module as out-of-process
- `executable_path` — path to the module binary (supports `~` expansion)
- `args` — command-line arguments passed to the executable
- `working_directory` — optional working directory for the process
- `environment` — environment variables to set for the process

## OoP Bootstrap Library

### Bootstrap entry point

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

### OopRunOptions fields

| Field | Description |
|-------|-------------|
| `module_name` | Logical module name (e.g., "file_parser") |
| `instance_id` | Instance ID (defaults to random UUID) |
| `directory_endpoint` | DirectoryService gRPC endpoint |
| `config_path` | Path to configuration file |
| `verbose` | Log verbosity (0=default, 1=info, 2=debug, 3=trace) |
| `print_config` | Print effective config and exit |
| `heartbeat_interval_secs` | Heartbeat interval (default: 5) |

## OoP Lifecycle

### Startup sequence

1. **Configuration loading** — loads config from file or `MODKIT_MODULE_CONFIG` env var
2. **Logging initialization** — sets up tracing with optional OTEL
3. **DirectoryService connection** — connects to the master host's directory service
4. **Instance registration** — registers with DirectoryService for discovery
5. **Heartbeat loop** — starts background heartbeat task
6. **Module lifecycle** — runs the normal module lifecycle (init → migrate → start)
7. **Graceful shutdown** — deregisters from DirectoryService on exit

### Shutdown model

Shutdown is driven by a single root `CancellationToken` per process:

- OS signals (SIGTERM, SIGINT, Ctrl+C) are hooked at bootstrap level
- The root token is passed to `RunOptions::Token` for module runtime shutdown
- Background tasks (like heartbeat) use child tokens derived from the root
- On shutdown, the module deregisters itself from DirectoryService before exiting

## SDK Pattern for OoP

### Module structure with SDK

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

### Key points

- The `-sdk` crate contains everything consumers need: API trait, types, gRPC client, and wiring helpers
- Server implementations are owned by the module itself, not the SDK
- Consumers only need one dependency: `my_module-sdk`

## SDK Crate Structure

### SDK `src/lib.rs`

```rust
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
use uuid::Uuid;

/// API trait for MyModule
#[async_trait]
pub trait MyModuleApi: Send + Sync {
    async fn do_something(&self, input: Input) -> Result<Output, MyModuleError>;
}

/// Input type
#[derive(Debug, Clone)]
pub struct Input {
    pub id: Uuid,
    pub message: String,
}

/// Output type
#[derive(Debug, Clone)]
pub struct Output {
    pub result: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Error type
#[derive(thiserror::Error, Debug)]
pub enum MyModuleError {
    #[error("Not found: {id}")]
    NotFound { id: Uuid },
    #[error("Validation error: {message}")]
    Validation { message: String },
    #[error("Internal error")]
    Internal,
}
```

### gRPC Client (in SDK)

```rust
// my_module-sdk/src/client.rs
use crate::{api::MyModuleApi, proto};
use async_trait::async_trait;
use modkit_transport_grpc::client::{GrpcClient, GrpcClientExt};
use tonic::transport::Channel;

pub struct MyModuleGrpcClient {
    inner: GrpcClient<proto::my_module_service_client::MyModuleServiceClient<Channel>>,
}

impl MyModuleGrpcClient {
    pub fn new(channel: Channel) -> Self {
        Self {
            inner: GrpcClient::new(proto::my_module_service_client::MyModuleServiceClient::new(channel)),
        }
    }
}

#[async_trait]
impl MyModuleApi for MyModuleGrpcClient {
    async fn do_something(&self, input: crate::api::Input) -> Result<crate::api::Output, crate::api::MyModuleError> {
        let request = proto::DoSomethingRequest {
            id: Some(input.id.to_string()),
            message: input.message,
        };

        let response = self
            .inner
            .call(|client| async move { client.do_something(request).await })
            .await
            .map_err(|e| crate::api::MyModuleError::Internal)?;

        Ok(crate::api::Output {
            result: response.result,
            timestamp: chrono::DateTime::parse_from_rfc3339(&response.timestamp)
                .map_err(|_| crate::api::MyModuleError::Internal)?
                .with_timezone(&chrono::Utc),
        })
    }
}
```

### Wiring helpers (in SDK)

```rust
// my_module-sdk/src/wiring.rs
use crate::{MyModuleApi, MyModuleGrpcClient, SERVICE_NAME};
use modkit_transport_grpc::client::{connect_with_stack, connect_with_retry};
use tonic::transport::Channel;

/// Wire a gRPC client with default stack
pub async fn wire_client(endpoint: &str) -> Result<Box<dyn MyModuleApi>, Box<dyn std::error::Error>> {
    let channel = connect_with_stack(endpoint).await?;
    let client = MyModuleGrpcClient::new(channel);
    Ok(Box::new(client))
}

/// Wire a gRPC client with retry logic
pub async fn build_client(
    endpoint: &str,
    max_retries: u32,
    retry_delay: std::time::Duration,
) -> Result<Box<dyn MyModuleApi>, Box<dyn std::error::Error>> {
    let channel = connect_with_retry(endpoint, max_retries, retry_delay).await?;
    let client = MyModuleGrpcClient::new(channel);
    Ok(Box::new(client))
}

/// Get service name for discovery
pub fn service_name() -> &'static str {
    SERVICE_NAME
}
```

## Module Implementation

### gRPC Server (in module)

```rust
// my_module/src/grpc_server.rs
use crate::api::{MyModuleApi, Input, Output, MyModuleError};
use crate::proto::{my_module_service_server::MyModuleService, DoSomethingRequest, DoSomethingResponse};
use async_trait::async_trait;
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct MyModuleGrpcServer {
    service: Arc<dyn MyModuleApi>,
}

impl MyModuleGrpcServer {
    pub fn new(service: Arc<dyn MyModuleApi>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl MyModuleService for MyModuleGrpcServer {
    async fn do_something(
        &self,
        request: Request<DoSomethingRequest>,
    ) -> Result<Response<DoSomethingResponse>, Status> {
        let req = request.into_inner();
        
        let input = Input {
            id: req.id.ok_or_else(|| Status::invalid_argument("id required"))?
                .parse()
                .map_err(|_| Status::invalid_argument("invalid id"))?,
            message: req.message,
        };

        match self.service.do_something(input).await {
            Ok(output) => {
                let response = DoSomethingResponse {
                    result: output.result,
                    timestamp: output.timestamp.to_rfc3339(),
                };
                Ok(Response::new(response))
            }
            Err(err) => Err(match err {
                MyModuleError::NotFound { .. } => Status::not_found(err.to_string()),
                MyModuleError::Validation { .. } => Status::invalid_argument(err.to_string()),
                MyModuleError::Internal => Status::internal(err.to_string()),
            }),
        }
    }
}
```

### Module registration with gRPC

```rust
// my_module/src/module.rs
#[modkit::module(
    name = "my_module",
    capabilities = [stateful],
    client = my_module_sdk::MyModuleApi,
    lifecycle(entry = "serve", stop_timeout = "30s")
)]
pub struct MyModule {
    service: Arc<crate::domain::service::MyService>,
}

impl MyModule {
    pub fn new() -> Self {
        Self {
            service: Arc::new(crate::domain::service::MyService::new()),
        }
    }

    async fn serve(
        self: Arc<Self>,
        cancel: CancellationToken,
        ready: ReadySignal,
    ) -> anyhow::Result<()> {
        // Create gRPC server
        let addr = "0.0.0.0:50051".parse()?;
        let grpc_server = MyModuleGrpcServer::new(self.service.clone());
        
        // Start server
        let server_future = tonic::transport::Server::builder()
            .add_service(my_module_sdk::proto::my_module_service_server::MyModuleServiceServer::new(grpc_server))
            .serve_with_shutdown(addr, cancel.cancelled());
        
        ready.notify();
        
        server_future.await?;
        Ok(())
    }
}
```

> The `client = ...` attribute validates the trait at compile time and exposes MODULE_NAME, but does not auto-register the client into ClientHub. You must still register it explicitly in your `init()` method using `ctx.client_hub().register::<dyn my_module_sdk::MyModuleApi>(client)`. 

## Client Registration (in gateway)

### Register both local and remote clients

```rust
// In gateway's init()
async fn register_clients(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
    // Try local client first
    if let Ok(local_client) = ctx.client_hub().try_get::<dyn my_module_sdk::MyModuleApi>() {
        ctx.client_hub().register::<dyn my_module_sdk::MyModuleApi>(local_client);
        return Ok(());
    }
    
    // Fall back to remote client
    let endpoint = "http://127.0.0.1:50051";
    let remote_client = my_module_sdk::wire_client(endpoint).await?;
    ctx.client_hub().register::<dyn my_module_sdk::MyModuleApi>(remote_client);
    
    Ok(())
}
```

## Testing OoP modules

### Test with mock server

```rust
#[tokio::test]
async fn test_grpc_client() {
    // Start mock server
    let mock_server = MockMyModuleServer::new();
    let server_addr = mock_server.start().await;
    
    // Create client
    let client = my_module_sdk::wire_client(&format!("http://{}", server_addr)).await.unwrap();
    
    // Test API
    let input = Input {
        id: Uuid::new_v4(),
        message: "test".to_string(),
    };
    
    let result = client.do_something(input).await.unwrap();
    assert!(!result.result.is_empty());
}
```

## Quick checklist

- [ ] Create `*-sdk` crate with API trait, types, gRPC client, and wiring helpers.
- [ ] Define `.proto` file and generate gRPC stubs in SDK.
- [ ] Implement gRPC server in module crate.
- [ ] Use `modkit_transport_grpc::client` utilities for connections.
- [ ] Register both local and remote clients in gateway.
- [ ] Use `CancellationToken` for coordinated shutdown.
- [ ] Test with mock gRPC servers.
