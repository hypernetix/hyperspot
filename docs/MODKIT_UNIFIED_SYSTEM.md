# ModKit — Architecture & Developer Guide (DDD-light)

This guide explains how to build production-grade modules on **ModKit**: how to lay out a module, declare it with a macro, wire REST with a type-safe builder, publish typed clients, and run background services with a clean lifecycle. It also describes the DDD-light layering and conventions used across modules.

---

## What ModKit provides

* **Composable modules** discovered via `inventory`, initialized in dependency order.
* **Ingress as a module** (e.g., `api_ingress`) that owns the Axum router and OpenAPI document.
* **Type-safe REST** via an operation builder that prevents half-wired routes at compile time.
* **Server-Sent Events (SSE)** with type-safe broadcasters and domain event integration.
* **OpenAPI 3.1** generation using `utoipa` with automatic schema registration for DTOs.
* **Standardized HTTP errors** with RFC-9457 `Problem` (implements `IntoResponse` directly).
* **Typed ClientHub** for in-process clients (resolve by interface type + optional scope).
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
  │  ├─ infra/                       # “low-level”: DB, system, IO, adapters
  │  │  ├─ storage/
  │  │  │  ├─ entity.rs              # e.g., SeaORM entities / SQL mappings
  │  │  │  ├─ mapper.rs              # entity <-> contract conversions (From impls)
  │  │  │  └─ migrations/
  │  │  │     ├─ mod.rs
  │  │  │     └─ initial_001.rs
  │  │  └─ (other platform adapters)
  │  └─ api/
  │     └─ rest/
  │        ├─ dto.rs                 # HTTP DTOs (serde/utoipa) — REST-only types
  │        ├─ handlers.rs            # Axum handlers (web controllers)
  │        └─ routes.rs              # route & OpenAPI registration (OperationBuilder)
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
}
```

### Common usage

**Typed config**

```rust
#[derive(serde::Deserialize, Default, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct MyModuleConfig { /* fields */ }
```

**DB access (SeaORM / SQLx)**

```rust
let sea = db.sea();      // SeaORM connection
let pool = db.sqlx_pool();  // SQLx pool
```

**Clients (publish & consume)**

```rust
// publish (provider module, in init()):
expose_my_module_client(&ctx, &api)?;

// consume (consumer module, in init()):
let api = my_module_client(&ctx.client_hub);
// or without helpers:
let api = ctx.client_hub.get::<dyn my_module::contract::client::MyModuleApi>()?;
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
pub struct MyModule { /* fields */ }
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
pub struct ApiIngress { /* ... */ }

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

## REST with `OperationBuilder`

`OperationBuilder` is a type-state builder that **won't compile** unless you set both a **handler** and at least one **response** before calling `register()`. It also attaches request bodies and component schemas using `utoipa`.

### Quick reference

**Constructors**

```rust
OperationBuilder::<Missing, Missing, S>::get("/path")
OperationBuilder::<Missing, Missing, S>::post("/path")
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
.allow_content_types(&["application/json", "application/xml"])
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
// Require authentication with resource:action permission:
.require_auth("users", "read")

// Or mark as public (no auth required):
.public()
```

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

Pass a state once via `Router::with_state(S)`. Handlers are free functions taking `State<S>`, so you don't capture/clone your service per route.

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
OperationBuilder::post("/users")
    .operation_id("users.create")
    .summary("Create user")
    .json_request::<CreateUserReq>(openapi, "User creation data")
    .handler(create_user_handler)
    .json_response_with_schema::<UserDto>(openapi, StatusCode::CREATED, "User created")
    .standard_errors(openapi)  // Adds 400, 401, 403, 404, 409, 422, 429, 500
    .register(router, openapi);

// Or for more specific error responses:
OperationBuilder::post("/users")
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

pub struct UserService { /* ... */ }

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

ModKit provides built-in support for Server-Sent Events through the `SseBroadcaster<T>` type and `OperationBuilder` integration. This enables real-time streaming of typed events to web clients with proper OpenAPI documentation.

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
) -> Sse<impl Stream<Item = Result<axum::response::sse::Event, Infallible>>> {
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
    OperationBuilder::<Missing, Missing, S>::get("/users/events")
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
OperationBuilder::post("/upload")
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
OperationBuilder::post("/upload")
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

Both helpers automatically configure MIME type validation via the ingress middleware. If a request arrives with a different Content-Type, it will receive HTTP 415 (Unsupported Media Type).

You can also manually configure allowed types:

```rust
OperationBuilder::post("/upload")
    .operation_id("files.upload_custom")
    .summary("Upload with custom validation")
    .allow_content_types(&["application/pdf", "image/png", "image/jpeg"])
    .handler(upload_handler)
    .json_response(StatusCode::OK, "Upload successful")
    .problem_response(openapi, StatusCode::UNSUPPORTED_MEDIA_TYPE, "Unsupported file type")
    .register(router, openapi);
```

**Important**: `.allow_content_types()` is independent of the request body schema. It only configures ingress validation and doesn't create OpenAPI request body specs. Use it when you want to enforce MIME types but handle the body parsing manually in your handler.

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

ModKit provides a standalone OpenAPI registry system that collects operation specs and schemas, then builds a complete OpenAPI 3.1 document.

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
let openapi = registry.build_openapi(&info)?;

// Serialize to JSON
let json = serde_json::to_string_pretty(&openapi)?;
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

The registry automatically adds a `bearerAuth` security scheme (HTTP Bearer with JWT format). Operations using `.require_auth()` will reference this scheme in OpenAPI.

### Content type handling

* `application/json` and `application/problem+json` reference component schemas
* `text/event-stream` (SSE) references event schemas
* `multipart/form-data` generates object schemas with binary file fields
* `application/octet-stream` generates string schemas with binary format
* Other content types use string schemas with custom formats

---

## Typed ClientHub

* **`contract::client`** defines the trait & DTOs exposed to other modules.
* **`gateways/local.rs`** implements that trait and is published in `init`.
* Consumers resolve the typed client from ClientHub by interface type (+ optional scope).

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
    fn as_any(&self) -> &dyn std::any::Any { self }
}
```

**Consume**

```rust
let api = my_module_client(&ctx.client_hub);
// or:
let api = ctx.client_hub.get::<dyn my_module::contract::client::MyModuleApi>()?;
```

---

## Contracts & lifecycle traits

```rust
#[async_trait::async_trait]
pub trait Module: Send + Sync + 'static {
    async fn init(&self, ctx: &crate::context::ModuleCtx) -> anyhow::Result<()>;
    fn as_any(&self) -> &dyn std::any::Any;
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
   Encodes **business rules/orchestration**. It calls repositories/infrastructure, applies invariants, aggregates data, owns retries/timeouts at the business level.

2. **Where to put “low-level” things?**
   In **infra/** (storage, system probes, processes, files, raw SQL, HTTP to other systems). Domain calls infra via small interfaces/constructors.

3. **Where to keep “glue”?**
   Glue that adapts domain to transport lives in **api/rest** (HTTP DTOs, handlers). Glue that adapts domain to **other modules** lives in **gateways/** (client implementations). DB mapping glue sits in **infra/storage**.

4. **Why not put platform-dependent logic into service?**
   To keep business rules portable/testable. Platform logic churns often; isolating it in infra avoids leaking OS/DB concerns into your domain.

5. **What is `contract` and why separate?**
   It’s the **public API** of your module for **other modules**: traits + DTOs + domain errors safe to expose. This separation allows swapping local/remote clients without changing consumers. For simple internal modules you may re-export a subset of domain models via `contract::model`.

6. **How to hide domain & internals from other modules?**
   Re-export only what’s needed via `contract`. Consumers depend on `contract` and `gateways` through the ClientHub; they never import your domain/infra directly.

---

## Complete example: Modern REST module

Here's a complete example showing all the latest features:

```rust
use axum::{body::Bytes, extract::{Multipart, Query, State}, Json, Router};
use http::StatusCode;
use modkit::api::{OpenApiRegistry, OperationBuilder, problem::{Problem, bad_request, internal_error}};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use std::sync::Arc;

// DTOs with utoipa schemas
#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreateItemRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ItemDto {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct RenderQuery {
    #[serde(default)]
    pub render_markdown: bool,
}

// State for handlers
#[derive(Clone)]
struct ApiState {
    service: Arc<ItemService>,
}

// Handlers (free functions using State)
async fn list_items(State(state): State<ApiState>) -> Result<Json<Vec<ItemDto>>, Problem> {
    let items = state.service.list_items().await
        .map_err(|e| internal_error(e))?;
    Ok(Json(items))
}

async fn create_item(
    State(state): State<ApiState>,
    Json(req): Json<CreateItemRequest>,
) -> Result<Json<ItemDto>, Problem> {
    if req.name.is_empty() {
        return Err(bad_request("Name is required"));
    }
    let item = state.service.create_item(req).await
        .map_err(|e| internal_error(e))?;
    Ok(Json(item))
}

async fn upload_file(
    State(state): State<ApiState>,
    Query(query): Query<RenderQuery>,
    mut multipart: Multipart,
) -> Result<Json<ItemDto>, Problem> {
    while let Some(field) = multipart.next_field().await.map_err(|e| internal_error(e))? {
        if field.name() == Some("file") {
            let filename = field.file_name().map(|s| s.to_string());
            let bytes = field.bytes().await.map_err(|e| internal_error(e))?;
            
            let item = state.service.process_upload(filename, bytes, query.render_markdown).await
                .map_err(|e| internal_error(e))?;
            return Ok(Json(item));
        }
    }
    Err(bad_request("Missing 'file' field"))
}

async fn upload_binary(
    State(state): State<ApiState>,
    body: Bytes,
) -> Result<Json<ItemDto>, Problem> {
    let item = state.service.process_binary(body).await
        .map_err(|e| internal_error(e))?;
    Ok(Json(item))
}

// Register routes using the type-safe builder
pub fn register_routes(
    mut router: Router,
    openapi: &dyn OpenApiRegistry,
    service: Arc<ItemService>,
) -> anyhow::Result<Router> {
    // GET /items - List items
    router = OperationBuilder::get("/items")
        .operation_id("items.list")
        .summary("List all items")
        .tag("Items")
        .require_auth("items", "read")
        .handler(list_items)
        .json_response_with_schema::<Vec<ItemDto>>(openapi, StatusCode::OK, "List of items")
        .standard_errors(openapi)
        .register(router, openapi);

    // POST /items - Create item
    router = OperationBuilder::post("/items")
        .operation_id("items.create")
        .summary("Create a new item")
        .tag("Items")
        .require_auth("items", "write")
        .json_request::<CreateItemRequest>(openapi, "Item data")
        .handler(create_item)
        .json_response_with_schema::<ItemDto>(openapi, StatusCode::CREATED, "Item created")
        .with_422_validation_error(openapi)
        .standard_errors(openapi)
        .register(router, openapi);

    // POST /items/upload - Upload file (multipart)
    router = OperationBuilder::post("/items/upload")
        .operation_id("items.upload")
        .summary("Upload a file")
        .tag("Items")
        .require_auth("items", "write")
        .query_param_typed("render_markdown", false, "Render markdown output", "boolean")
        .multipart_file_request("file", Some("File to process"))
        .handler(upload_file)
        .json_response_with_schema::<ItemDto>(openapi, StatusCode::OK, "File processed")
        .standard_errors(openapi)
        .problem_response(openapi, StatusCode::UNSUPPORTED_MEDIA_TYPE, "Unsupported file type")
        .register(router, openapi);

    // POST /items/upload-binary - Upload raw binary
    router = OperationBuilder::post("/items/upload-binary")
        .operation_id("items.upload_binary")
        .summary("Upload raw binary data")
        .tag("Items")
        .require_auth("items", "write")
        .octet_stream_request(Some("Raw file bytes"))
        .handler(upload_binary)
        .json_response_with_schema::<ItemDto>(openapi, StatusCode::OK, "Binary processed")
        .standard_errors(openapi)
        .register(router, openapi);

    // Add state
    let state = ApiState { service };
    Ok(router.with_state(state))
}
```

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
