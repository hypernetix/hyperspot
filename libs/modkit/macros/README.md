# ModKit Macros

Procedural macros for the ModKit framework, focused on generating strongly-typed gRPC clients with built-in SecurityCtx propagation.

## Overview

ModKit provides two macros for generating gRPC client implementations:

1. **`#[generate_clients]`** (RECOMMENDED) - Generate a gRPC client from an API trait definition with automatic SecurityCtx propagation
2. **`#[grpc_client]`** - Generate a gRPC client with manual trait implementation

## Quick Start

### Recommended: Using `generate_clients`

The `generate_clients` macro is applied to your API trait and automatically generates a strongly-typed gRPC client with full method delegation and automatic SecurityCtx propagation:

```rust
use modkit_macros::generate_clients;
use modkit_security::SecurityCtx;

#[generate_clients(
    grpc_client = "modkit_users_v1::users_service_client::UsersServiceClient<tonic::transport::Channel>"
)]
#[async_trait::async_trait]
pub trait UsersApi: Send + Sync {
    async fn get_user(&self, ctx: &SecurityCtx, req: GetUserRequest) 
        -> Result<UserResponse, UsersError>;
    
    async fn list_users(&self, ctx: &SecurityCtx, req: ListUsersRequest) 
        -> Result<Vec<UserResponse>, UsersError>;
}
```

This generates:

- The original `UsersApi` trait (unchanged)
- `UsersApiGrpcClient` - wraps the tonic client with:
  - Automatic proto ↔ domain type conversions
  - Automatic SecurityCtx propagation via gRPC metadata
  - Standard transport stack (timeouts, retries, metrics, tracing)

The client fully implements the `UsersApi` trait with automatic method delegation.

#### Usage

```rust
// Connect to gRPC service
let client = UsersApiGrpcClient::connect("http://localhost:50051").await?;

// SecurityCtx is automatically propagated via gRPC metadata
let ctx = SecurityCtx::for_user(user_id);
let user = client.get_user(&ctx, GetUserRequest { id: "123" }).await?;

// Or with custom configuration
let config = GrpcClientConfig::new("users_service")
    .with_connect_timeout(Duration::from_secs(5))
    .with_rpc_timeout(Duration::from_secs(15));
    
let client = UsersApiGrpcClient::connect_with_config(
    "http://localhost:50051",
    &config
).await?;
```

### Alternative: Manual `#[grpc_client]`

If you need more control, you can use the `grpc_client` macro which generates the struct and helpers, but requires manual trait implementation:

```rust
use modkit_macros::grpc_client;

#[grpc_client(
    api = "crate::contracts::UsersApi",
    tonic = "modkit_users_v1::users_service_client::UsersServiceClient<tonic::transport::Channel>",
    package = "modkit.users.v1"
)]
pub struct UsersGrpcClient;

// You must manually implement the trait
#[async_trait::async_trait]
impl UsersApi for UsersGrpcClient {
    async fn get_user(&self, req: GetUserRequest) -> anyhow::Result<UserResponse> {
        let mut client = self.inner_mut();
        let request = tonic::Request::new(req.into());
        let response = client.get_user(request).await?;
        Ok(response.into_inner().into())
    }
    // ... other methods
}
```

## Local (In-Process) Clients

**Note:** ModKit no longer provides macro-based local client generation. For local (in-process) communication:

- **Recommended:** Register your service directly in `ClientHub` as `Arc<dyn YourTrait>`
- No wrapper or client struct is needed for local communication
- This provides zero-overhead, direct method calls

Example:

```rust
// Register service directly in ClientHub
let service: Arc<dyn UsersApi> = Arc::new(UsersService::new());
client_hub.register::<dyn UsersApi>(service);

// Retrieve and use
let api = client_hub.get::<dyn UsersApi>()?;
let user = api.get_user(request).await?;
```

## API Requirements

All API traits used with these macros must follow strict signature rules:

1. **Async methods**: All trait methods must be `async`
2. **Standard receiver**: Methods must use `&self` (not `&mut self` or `self`)
3. **Result return type**: Methods must return `Result<T, E>` with two type parameters
4. **Parameter patterns**: Methods must use one of two patterns:

### Pattern 1: Secured API (with SecurityCtx)

For APIs that require authorization and access control:

```rust
async fn method_name(
    &self,
    ctx: &SecurityCtx,
    req: RequestType,
) -> Result<ResponseType, ErrorType>;
```

The `SecurityCtx` parameter:
- Must be the **first** parameter after `&self`
- Must be an immutable reference (`&SecurityCtx`, not `&mut SecurityCtx`)
- The type must be named `SecurityCtx` (from `modkit_security::SecurityCtx` or aliased)

### Pattern 2: Unsecured API (without SecurityCtx)

For system-internal APIs that don't require user authorization:

```rust
async fn method_name(
    &self,
    req: RequestType,
) -> Result<ResponseType, ErrorType>;
```

### Valid Secured API Trait

```rust
use modkit_security::SecurityCtx;

#[async_trait::async_trait]
pub trait MyApi: Send + Sync {
    async fn get_item(&self, ctx: &SecurityCtx, req: GetItemRequest) 
        -> Result<ItemResponse, MyError>;
    
    async fn list_items(&self, ctx: &SecurityCtx, req: ListItemsRequest) 
        -> Result<Vec<ItemResponse>, MyError>;
}
```

### Valid Unsecured API Trait

```rust
#[async_trait::async_trait]
pub trait SystemApi: Send + Sync {
    async fn resolve_service(&self, name: String) 
        -> Result<Endpoint, SystemError>;
}
```

### How SecurityCtx Propagates

For secured APIs (with `ctx: &SecurityCtx`), the generated gRPC client:

1. **Client-side**: Serializes the `SecurityCtx` into gRPC metadata headers before sending the request
2. **Server-side**: The gRPC server extracts the `SecurityCtx` from metadata and passes it to your service
3. **Automatic**: No manual header management required

Example generated code:

```rust
async fn get_user(&self, ctx: &SecurityCtx, req: GetUserRequest) 
    -> Result<UserResponse, UsersError> 
{
    let mut client = self.inner.clone();
    let mut request = tonic::Request::new(req.into());
    
    // Automatically attach SecurityCtx to gRPC metadata
    modkit_transport_grpc::attach_secctx(request.metadata_mut(), ctx)?;
    
    let response = client.get_user(request).await?;
    Ok(response.into_inner().into())
}
```

### Invalid API Traits

```rust
// ❌ NOT async
fn get_item(&self, req: GetItemRequest) -> anyhow::Result<ItemResponse>;

// ❌ Multiple parameters after request
async fn get_item(&self, ctx: &SecurityCtx, id: String, name: String) 
    -> anyhow::Result<ItemResponse>;

// ❌ Wrong parameter order (request before ctx)
async fn get_item(&self, req: GetItemRequest, ctx: &SecurityCtx) 
    -> anyhow::Result<ItemResponse>;

// ❌ Mutable SecurityCtx reference
async fn get_item(&self, ctx: &mut SecurityCtx, req: GetItemRequest) 
    -> anyhow::Result<ItemResponse>;

// ❌ Not returning Result
async fn get_item(&self, req: GetItemRequest) -> ItemResponse;

// ❌ Mutable receiver
async fn get_item(&mut self, req: GetItemRequest) -> anyhow::Result<ItemResponse>;
```

## Generated Code Structure

Given a trait `UsersApi`, the `generate_clients` macro generates:

```rust
// Original trait (unchanged)
#[async_trait::async_trait]
pub trait UsersApi: Send + Sync {
    async fn get_user(&self, req: GetUserRequest) -> anyhow::Result<UserResponse>;
}

// gRPC client struct
pub struct UsersApiGrpcClient {
    inner: UsersServiceClient<tonic::transport::Channel>,
}

impl UsersApiGrpcClient {
    /// Connect with default configuration
    pub async fn connect(uri: impl Into<String>) -> anyhow::Result<Self> { /* ... */ }
    
    /// Connect with custom configuration
    pub async fn connect_with_config(
        uri: impl Into<String>,
        cfg: &GrpcClientConfig
    ) -> anyhow::Result<Self> { /* ... */ }
    
    /// Create from an existing channel
    pub fn from_channel(channel: tonic::transport::Channel) -> Self { /* ... */ }
}

#[async_trait::async_trait]
impl UsersApi for UsersApiGrpcClient {
    async fn get_user(&self, req: GetUserRequest) -> anyhow::Result<UserResponse> {
        let mut client = self.inner.clone();
        let request = tonic::Request::new(req.into());
        let response = client.get_user(request).await?;
        Ok(response.into_inner().into())
    }
}
```

## Transport Stack

All generated gRPC clients automatically use the standardized transport stack from `modkit-transport-grpc`, which provides:

- **Configurable timeouts**: Separate timeouts for connection establishment and individual RPC calls
- **Retry logic**: Automatic retry with exponential backoff for transient failures
- **Metrics collection**: Built-in Prometheus metrics for monitoring
- **Distributed tracing**: OpenTelemetry integration for request tracing

### Default Configuration

- Connect timeout: 10 seconds
- RPC timeout: 30 seconds
- Max retries: 3 attempts
- Base backoff: 100ms
- Max backoff: 5 seconds
- Metrics and tracing: Enabled

### Custom Configuration

```rust
use modkit_transport_grpc::client::GrpcClientConfig;

let config = GrpcClientConfig::new("my_service")
    .with_connect_timeout(Duration::from_secs(5))
    .with_rpc_timeout(Duration::from_secs(15))
    .with_max_retries(5)
    .without_metrics();

let client = UsersApiGrpcClient::connect_with_config(
    "http://localhost:50051",
    &config
).await?;
```

### Bypassing the Transport Stack

For testing or custom channel setup:

```rust
let channel = Channel::from_static("http://localhost:50051")
    .connect()
    .await?;

let client = UsersApiGrpcClient::from_channel(channel);
```

## Type Conversions

The generated gRPC client requires:

- Each request type `Req` implements `Into<ProtoReq>` where `ProtoReq` is the corresponding protobuf message
- Each response type `Resp` implements `From<ProtoResp>` where `ProtoResp` is the tonic response message

Example:

```rust
// Domain type
pub struct GetUserRequest {
    pub id: String,
}

// Conversion to protobuf
impl From<GetUserRequest> for proto::GetUserRequest {
    fn from(req: GetUserRequest) -> Self {
        proto::GetUserRequest { id: req.id }
    }
}

// Response conversion
impl From<proto::UserResponse> for UserResponse {
    fn from(proto: proto::UserResponse) -> Self {
        UserResponse {
            id: proto.id,
            name: proto.name,
        }
    }
}
```

If these conversions are missing, the code will not compile (by design).

## Best Practices

1. **Use `generate_clients` when possible** - It provides the most automated experience
2. **Keep API traits focused** - Each trait should represent a cohesive set of operations
3. **Use descriptive names** - Client structs are named after your trait (e.g., `UsersApi` → `UsersApiGrpcClient`)
4. **Implement type conversions** - Ensure domain types convert to/from protobuf
5. **Leverage trait objects** - Enables polymorphism via `Arc<dyn YourTrait>`

## Troubleshooting

### "generate_clients requires `grpc_client` parameter"

Ensure you provide the `grpc_client` parameter:

```rust
#[generate_clients(
    grpc_client = "path::to::TonicClient<Channel>"
)]
```

### "API methods must be async"

All trait methods must be marked `async`.

### "API methods must have exactly one parameter (besides &self)"

If you have multiple parameters, wrap them in a request struct:

```rust
// Instead of this:
async fn update(&self, id: String, name: String) -> Result<(), Error>;

// Use this:
#[derive(Clone)]
pub struct UpdateRequest {
    pub id: String,
    pub name: String,
}

async fn update(&self, req: UpdateRequest) -> Result<(), Error>;
```

### Missing Into/From implementations

Ensure you implement the required conversions between domain and proto types.

## See Also

- [ModKit Documentation](../../docs/)
- [API Guidelines](../../guidelines/API_GUIDELINE.md)
- [Module Creation](../../guidelines/NEW_MODULE.md)
