# ModKit Macros

Procedural macros for the ModKit framework, providing code generation for gRPC clients and REST API DTOs.

## Overview

ModKit provides three main macros:

### gRPC Client Generation

1. **`#[generate_clients]`** (RECOMMENDED) - Generate a gRPC client from an API trait definition with automatic SecurityCtx propagation
2. **`#[grpc_client]`** - Generate a gRPC client with manual trait implementation

### REST API DTO Generation

3. **`#[api_dto]`** - Generate REST API Data Transfer Objects with automatic serialization, schema generation, and type safety

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

---

## REST API DTO Macro

The `#[api_dto]` macro simplifies the creation of REST API Data Transfer Objects by automatically deriving serialization traits and OpenAPI schema generation.

### Quick Start

```rust
use modkit_macros::api_dto;

#[api_dto(request)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
}

#[api_dto(response)]
pub struct UserResponse {
    pub id: String,
    pub name: String,
    pub email: String,
}
```

### Macro Arguments

The `api_dto` macro accepts one or both of the following flags:

- **`request`** - Marks the type as a request DTO (implements `Deserialize` and `RequestApiDto`)
- **`response`** - Marks the type as a response DTO (implements `Serialize` and `ResponseApiDto`)

You can use both flags for types that serve as both request and response:

```rust
#[api_dto(request, response)]
pub struct UserDto {
    pub id: String,
    pub name: String,
}
```

### Generated Code

The macro automatically adds:

1. **Serde traits**: `Serialize` and/or `Deserialize` based on flags
2. **OpenAPI schema**: `utoipa::ToSchema` for automatic API documentation
3. **Snake case conversion**: `#[serde(rename_all = "snake_case")]` for consistent JSON formatting
4. **Marker traits**: `RequestApiDto` and/or `ResponseApiDto` for type safety

#### Example: Request DTO

```rust
#[api_dto(request)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
}
```

Expands to:

```rust
#[derive(serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
}

impl ::modkit::api::api_dto::RequestApiDto for CreateUserRequest {}
```

#### Example: Response DTO

```rust
#[api_dto(response)]
pub struct UserResponse {
    pub id: String,
    pub name: String,
}
```

Expands to:

```rust
#[derive(serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct UserResponse {
    pub id: String,
    pub name: String,
}

impl ::modkit::api::api_dto::ResponseApiDto for UserResponse {}
```

### Usage in REST APIs

Use `api_dto` types with the ModKit REST API framework:

```rust
use modkit::api::RestApiCapability;
use modkit_macros::api_dto;

#[api_dto(request)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
}

#[api_dto(response)]
pub struct UserResponse {
    pub id: String,
    pub name: String,
    pub email: String,
    pub created_at: String,
}

// Use in your REST API handler
async fn create_user(
    req: CreateUserRequest,
) -> Result<UserResponse, Error> {
    // Your implementation
    Ok(UserResponse {
        id: "123".to_string(),
        name: req.name,
        email: req.email,
        created_at: "2024-01-01T00:00:00Z".to_string(),
    })
}
```

### Validation and Error Handling

The macro requires at least one flag (`request` or `response`):

```rust
// ❌ Compile error: requires at least one flag
#[api_dto()]
pub struct InvalidDto {
    pub field: String,
}

// ✅ Valid
#[api_dto(request)]
pub struct ValidDto {
    pub field: String,
}
```

Unknown flags are rejected at compile time:

```rust
// ❌ Compile error: unknown flag 'invalid'
#[api_dto(invalid)]
pub struct MyDto {
    pub field: String,
}
```

Duplicate flags are also rejected:

```rust
// ❌ Compile error: duplicate flag 'request'
#[api_dto(request, request)]
pub struct MyDto {
    pub field: String,
}
```

### Benefits

1. **Type Safety**: Marker traits (`RequestApiDto`, `ResponseApiDto`) enable compile-time checks to ensure that types used for request/response in OperationBuilder will be compliant
2. **Consistency**: Automatic snake_case conversion ensures consistent JSON formatting
3. **Less Boilerplate**: No need to manually derive serde traits and configure attributes

### Best Practices

1. **Use specific flags**: Only add `request` or `response` based on actual usage
2. **Keep DTOs simple**: DTOs should be data containers without business logic
3. **Separate concerns**: Use different types for requests and responses when they differ

## See Also

- [ModKit Documentation](../../docs/)
- [API Guidelines](../../guidelines/API_GUIDELINE.md)
- [Module Creation](../../guidelines/NEW_MODULE.md)
