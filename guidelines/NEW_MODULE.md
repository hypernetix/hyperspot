# New Module Guideline (Hyperspot / ModKit)

This guide provides a comprehensive, step-by-step process for creating production-grade Hyperspot modules. It is
designed to be actionable for both human developers and LLM-based code generators, consolidating best practices from
across the Hyperspot ecosystem.

## ModKit Core Concepts

ModKit provides a powerful framework for building production-grade modules:

- **Composable Modules**: Discovered via `inventory` and initialized in dependency order.
- **Ingress as a Module**: `api_ingress` owns the Axum router and OpenAPI document.
- **Type-Safe REST**: An operation builder prevents half-wired routes at compile time.
- **Server-Sent Events (SSE)**: Type-safe broadcasters for real-time domain event integration.
- **Standardized HTTP Errors**: Built-in support for RFC-9457 `Problem` and `ProblemResponse`.
- **Typed ClientHub**: For in-process clients, resolved by interface type.
- **Lifecycle Management**: Helpers for long-running tasks and graceful shutdown.

## HyperSpot Modular architecture

A module is a composable unit implementing typically some business logic with either REST API and/or peristent storage.
Common and stateless logic that can be reusable across modules should be implemented in the `libs` crate.

## Canonical Project Layout

Modules follow a DDD-light architecture to ensure a clean separation of concerns:

- **`contract`**: The public API surface for inter-module communication. Transport-agnostic.
- **`domain`**: The core business logic and rules, independent of any infrastructure.
- **`infra`**: Implementations of external concerns like databases, message queues, or other system interactions.
- **`api`**: Adapters that translate transport-level requests (e.g., REST) to domain-level commands.

All modules MUST adhere to the following directory structure:

```
modules/<your-module>/
  src/
    lib.rs              # Public exports (contract, module type), hides internals
    module.rs           # Module struct, #[modkit::module], and trait impls
    config.rs           # Typed config with defaults
    api/                # Transport adapters
      mod.rs            # Re-exports for api
      rest/             # HTTP REST layer
        mod.rs          # Re-exports for rest
        dto.rs          # DTOs (serde, ToSchema)
        handlers.rs     # Thin Axum handlers
        mapper.rs       # From/Into DTO<->Model conversions
        routes.rs       # OperationBuilder registrations
        error.rs        # ProblemResponse mapping
        sse_adapter.rs  # SSE event publisher adapter (optional)
    contract/           # Public API for other modules (NO serde)
      mod.rs            # Re-exports for contract submodules
      client.rs         # Native ClientHub trait (<Module>Api)
      model.rs          # Transport-agnostic models
      error.rs          # Transport-agnostic errors
    domain/             # Internal business logic
      mod.rs            # Re-exports for domain
      events.rs         # Domain events (transport-agnostic)
      ports.rs          # Output ports (e.g., EventPublisher)
      repository.rs     # Repository traits (ports)
      service.rs        # Service orchestrating business logic
    infra/              # Infrastructure adapters
      mod.rs            # Re-exports for infra
      storage/          # Database layer
        entity.rs       # SeaORM entities
        mapper.rs       # From/Into Model<->Entity conversions
        repositories.rs # SeaORM repository implementations
        migrations/     # SeaORM migrations
          mod.rs        # Migrator module entry
    gateways/           # Adapters for client traits
      mod.rs            # Re-exports for gateways
      local.rs          # Local client implementing contract API
  Cargo.toml
```

---

## Step-by-Step Generation Guide

> Note: Strictly mirror the style, naming, and structure of the `examples/modkit/users_info/` reference when generating
> code.

### Step 1: Project & Cargo Setup

1. **Create `Cargo.toml`:**
   **Rule:** Dependencies and features MUST mirror the canonical `users_info` example to ensure workspace compatibility.

   ```toml
   [package]
   name = "<your-module-name>"
   version.workspace = true
   publish = false
   edition.workspace = true
   license.workspace = true
   authors.workspace = true

   [dependencies]
   anyhow = { workspace = true }
   async-trait = { workspace = true }
   tokio = { workspace = true }
   tracing = { workspace = true }
   inventory = { workspace = true }
   serde = { workspace = true }
   serde_json = { workspace = true }
   utoipa = { workspace = true }
   axum = { workspace = true, features = ["macros"] }
   tower-http = { workspace = true, features = ["timeout"] }
   futures = { workspace = true }
   chrono = { workspace = true, features = ["serde"] }
   uuid = { workspace = true, features = ["v4", "serde"] }
   arc-swap = { workspace = true }
   sea-orm = { workspace = true, features = ["sqlx-sqlite", "runtime-tokio-rustls", "macros", "with-chrono", "with-uuid"] }
   sea-orm-migration = { workspace = true }
   thiserror = { workspace = true }
   modkit = { path = "../../../libs/modkit" }
   db = { path = "../../../libs/db" }

   [dev-dependencies]
   tower = { workspace = true, features = ["util"] }
   api_ingress = { path = "../../../modules/api_ingress" }
   ```

2. **Create `src/lib.rs`:**

   **Rule:** The public API surface MUST be limited to the `contract` and the main module type. All other modules (
   `api`, `domain`, `infra`, etc.) MUST be marked with `#[doc(hidden)]`.

   ```rust
   // === PUBLIC CONTRACT ===
   pub mod contract;
   pub use contract::{client, error, model};

   // === MODULE DEFINITION ===
   pub mod module;
   pub use module::<YourModuleTypeName>; // Replace with your module's struct name

   // === INTERNAL MODULES ===
   #[doc(hidden)] pub mod api;
   #[doc(hidden)] pub mod config;
   #[doc(hidden)] pub mod domain;
   #[doc(hidden)] pub mod gateways;
   #[doc(hidden)] pub mod infra;
   ```

### Step 2: Data types naming matrix

**Rule:** Use the following naming matrix for your data types:

| Operation              | DB Layer (sqlx/SeaORM)<br/>`src/infra/storage/entity.rs` | Domain Layer (contract model)<br/>`src/contract/model.rs` | API Request (in)<br/>`src/api/rest/dto.rs`      | API Response (out)<br/>`src/api/rest/dto.rs`                                                    |
|------------------------|----------------------------------------------------------|-----------------------------------------------------------|-------------------------------------------------|-------------------------------------------------------------------------------------------------|
| Create                 | ActiveModel                                              | NewUser                                                   | CreateUserRequest                               | UserResponse                                                                                    |
| Read/Get by id         | UserEntity                                               | User                                                      | Path params (id)<br/>`routes.rs` registers path | UserResponse                                                                                    |
| List/Query             | UserEntity (rows)                                        | User (Vec/User iterator)                                  | ListUsersQuery (filter+page)                    | UserListResponse or Page<UserView>                                                              |
| Update (PUT, full)     | UserEntity (update query)                                | UpdatedUser (optional)                                    | UpdateUserRequest                               | UserResponse                                                                                    |
| Patch (PATCH, partial) | UserPatchEntity (optional)                               | UserPatch                                                 | PatchUserRequest                                | UserResponse                                                                                    |
| Delete                 | (no payload)                                             | DeleteUser (optional command)                             | Path params (id)<br/>`routes.rs` registers path | NoContent (204) or DeleteUserResponse (rare)<br/>`handlers.rs` return type + `error.rs` mapping |
| Search (text)          | UserSearchEntity (projection)                            | UserSearchHit                                             | SearchUsersQuery                                | SearchUsersResponse (hits + meta)                                                               |
| Projection/View        | UserAggEntity / UserSummaryEntity                        | UserSummary                                               | (n/a)                                           | UserSummaryView                                                                                 |

Notes:

- Keep all transport-agnostic types in `src/contract/model.rs`. Handlers and DTOs must not leak into `contract`.
- SeaORM entities live in `src/infra/storage/entity.rs` (or submodules). Repository queries go in
  `src/infra/storage/repositories.rs`.
- All REST DTOs (requests/responses/views) live in `src/api/rest/dto.rs`; provide `From` conversions in
  `src/api/rest/mapper.rs`.

### Step 3: Errors management

#### Errors definition

**Rule:** Use the following naming and placement matrix for error types and mappings:

| Concern                        | Type/Concept                                   | File (must define)         | Notes                                                                                                                                        |
|--------------------------------|------------------------------------------------|----------------------------|----------------------------------------------------------------------------------------------------------------------------------------------|
| Domain error (business)        | `DomainError`                                  | `src/domain/error.rs`      | Pure business errors; no transport details. Variants reflect domain invariants (e.g., `UserNotFound`, `EmailAlreadyExists`, `InvalidEmail`). |
| Contract error (public)        | `<ModuleName>Error`                            | `src/contract/error.rs`    | Transport-agnostic surface for other modules. Provide `From<DomainError> for <ModuleName>Error`. No `serde` derives.                         |
| REST mapping function          | `map_domain_error(...) -> ProblemResponse`     | `src/api/rest/error.rs`    | Centralize RFC-9457 mapping: choose status, title, detail; include `.with_instance(path)`.                                                   |
| Handler usage                  | `map_domain_error(&e, uri.path())`             | `src/api/rest/handlers.rs` | Always map errors via the centralized function; return `ProblemResponse`.                                                                    |
| OpenAPI responses registration | `.problem_response(openapi, <status>, <desc>)` | `src/api/rest/routes.rs`   | Register all error statuses your handler can return to keep OpenAPI in sync.                                                                 |

Error design rules:

- Use situation-specific error structs (not mega-enums); include `Backtrace` where helpful.
- Provide convenience `is_xxx()` helper methods on error types.

Recommended error variant mapping (example for Users):

| DomainError variant         | Contract error variant | HTTP status | Problem title    | Detail                          |
|-----------------------------|------------------------|-------------|------------------|---------------------------------|
| `UserNotFound(id)`          | `NotFound(id)`         | 404         | "User not found" | `No user with id {id}`          |
| `EmailAlreadyExists(email)` | `EmailConflict(email)` | 409         | "Conflict"       | `Email already exists: {email}` |                       |                       |

Minimal templates:

```rust
// src/domain/error.rs
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("User not found: {0}")] UserNotFound(uuid::Uuid),
    #[error("Email already exists: {0}")] EmailAlreadyExists(String),
}
```

```rust
// src/contract/error.rs
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum UsersInfoError {
    #[error("User not found with ID: {0}")] NotFound(uuid::Uuid),
    #[error("Email already exists: {0}")] EmailConflict(String),
}

impl From<crate::domain::error::DomainError> for UsersInfoError {
    fn from(e: crate::domain::error::DomainError) -> Self {
        match e {
            crate::domain::error::DomainError::UserNotFound(id) => Self::NotFound(id),
            crate::domain::error::DomainError::EmailAlreadyExists(email) => Self::EmailConflict(email),        }
    }
}
```

```rust
// src/api/rest/error.rs
use modkit::api::problem::{Problem, ProblemResponse};
use crate::contract::error::UsersInfoError;

/// Helper to create a ProblemResponse with less boilerplate
pub fn from_parts(
    status: StatusCode,
    code: &str,
    title: &str,
    detail: impl Into<String>,
    instance: &str,
) -> ProblemResponse {
    let problem = Problem::new(status, title, detail)
        .with_type(format!("https://errors.hyperspot.com/{}", code))
        .with_code(code)
        .with_instance(instance);

    // Add request ID from current tracing span if available
    let problem = if let Some(id) = tracing::Span::current().id() {
        problem.with_trace_id(id.into_u64().to_string())
    } else {
        problem
    };

    ProblemResponse(problem)
}

pub fn map_domain_error(err: &UsersInfoError, instance_path: &str) -> ProblemResponse {
    match err {
        UsersInfoError::NotFound(id) => from_parts(
            StatusCode::NOT_FOUND,
            "USERS_NOT_FOUND",
            "User not found",
            format!("No user with id {}", id),
            instance_path,
        ),
        UsersInfoError::EmailConflict(email) => from_parts(
            StatusCode::CONFLICT,
            "USERS_EMAIL_CONFLICT",
            "Email already exists",
            format!("Email already exists: {}", email),
            instance_path,
        ),
    }
}
```

#### The Problem object usage

**Rule:** Always use the Problem object to create ProblemResponse objects.

**Rule:** The Problem object creation:

```rust
let problem = Problem::new(status_code, title, detail)
.with_type(format!("https://errors.hyperspot.com/{}", code))
.with_code(code)
.with_instance(instance_path);

// Add tracing span ID if available
let problem = if let Some(id) = tracing::Span::current().id() {
problem.with_trace_id(id.into_u64().to_string())
} else {
problem
};

ProblemResponse(problem)
```

#### Error Conversion Chain

**Rule:** Always convert domain errors to contract errors before mapping to REST:

```rust
// In handlers.rs - CORRECT pattern
.map_err( | e: DomainError| map_domain_error( & e.into(), uri.path())) ?

// Or with explicit type annotation when compiler needs help
.map_err( | e: SysCapError| map_domain_error( & e, uri.path())) ?
```

Checklist:

- Provide `From<DomainError> for <Module>Error`.
- Use `map_domain_error` in all handlers.
- Register `.problem_response(openapi, 400/404/409/500, ...)` for each route as applicable.
- Keep all contract errors free of `serde` and any transport specifics.
- Validation errors SHOULD use `422 Unprocessable Entity` when applicable; otherwise use `400 Bad Request`.
- **Always use the `from_parts` helper function for consistent Problem creation**.

### Step 4: Contract Layer (Public API for Rust clients)

This layer defines the transport-agnostic interface for your module.

Contract API design rules:

- Do not expose smart pointers (`Arc<T>`, `Box<T>`) in public APIs.
- Accept `impl AsRef<str>` instead of `&str` for flexibility.
- Accept `impl AsRef<Path>` for file paths.
- Use inherent methods for core functionality; use traits for extensions.
- Public contract types MUST implement `Debug`. Types intended for display SHOULD implement `Display`.

1. **Clean Contract**: `contract` models and errors MUST NOT have `serde` or any other transport-specific derives.

2. **`src/contract/model.rs`:**
   **Rule:** Contract models MUST NOT have `serde` or any other transport-specific derives. They are plain Rust structs
   for inter-module communication.

   ```rust
   // Example from users_info
   #[derive(Debug, Clone, PartialEq, Eq)]
   pub struct User {
       pub id: uuid::Uuid,
       pub email: String,
       pub display_name: String,
       pub created_at: chrono::DateTime<chrono::Utc>,
       pub updated_at: chrono::DateTime<chrono::Utc>,
   }
   ```

3. **`src/contract/error.rs`:**
   **Rule:** Define a domain-specific error enum for the contract. This allows other modules to handle your errors
   without depending on implementation details.

   ```rust
   // Example from users_info
   #[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
   pub enum UsersInfoError {
       #[error("User not found with ID: {0}")]
       NotFound(uuid::Uuid),
       #[error("Email already exists: {0}")]
       EmailConflict(String),
       #[error("Invalid input: {0}")]
       Validation(String),
       #[error("An internal database error occurred")]
       Database,
   }

   // Provide From conversions between domain and contract errors
   impl From<crate::domain::error::DomainError> for UsersInfoError {
       fn from(e: crate::domain::error::DomainError) -> Self {
           match e {
               crate::domain::error::DomainError::UserNotFound(id) => Self::NotFound(id),
               crate::domain::error::DomainError::EmailAlreadyExists(email) => Self::EmailConflict(email),
               crate::domain::error::DomainError::InvalidEmail(_) => Self::Validation("Invalid email format".to_string()),
               _ => Self::Database,
           }
       }
   }
   ```

4. **`src/contract/client.rs`:**
   **Rule:** Define a native, async trait for the ClientHub. This is the primary way other modules will interact with
   your service. Name it `<PascalCaseModule>Api`.

   ```rust
   // Example from users_info
   #[async_trait::async_trait]
   pub trait UsersInfoApi: Send + Sync {
       async fn get_user(&self, id: uuid::Uuid) -> Result<super::model::User, super::error::UsersInfoError>;
       // ... other methods
   }
   ```

### Step 5: Domain Layer

This layer contains the core business logic, free from API specifics andinfrastructure concerns.

1. **`src/domain/events.rs`:**
   **Rule:** Define transport-agnostic domain events for important business actions.

   ```rust
   // Example from users_info
   #[derive(Debug, Clone)]
   pub enum UserDomainEvent {
       Created { id: uuid::Uuid, at: chrono::DateTime<chrono::Utc> },
       Updated { id: uuid::Uuid, at: chrono::DateTime<chrono::Utc> },
       Deleted { id: uuid::Uuid, at: chrono::DateTime<chrono::Utc> },
   }
   ```

2. **`src/domain/ports.rs`:**
   **Rule:** Define output ports (interfaces) for external concerns like event publishing.

   ```rust
   // Example from users_info
   pub trait EventPublisher<E>: Send + Sync + 'static {
       fn publish(&self, event: &E);
   }
   ```

3. **`src/domain/repository.rs`:**
   **Rule:** Define repository traits (ports) that the service will depend on. This decouples the domain from the
   database implementation.

   **Rule:** For ranged queries, prefer accepting `impl core::ops::RangeBounds<T>` parameters to make callers flexible
   while keeping type safety.

   ```rust
   // Example from users_info
   #[async_trait::async_trait]
   pub trait UsersRepository: Send + Sync {
       async fn find_by_id(&self, id: uuid::Uuid) -> anyhow::Result<Option<crate::contract::model::User>>;
       async fn email_exists(&self, email: &str) -> anyhow::Result<bool>;
       // ... other data access methods
       // Example of ranged queries using RangeBounds
       async fn list_by_created_at<R>(
           &self,
           range: R,
       ) -> anyhow::Result<Vec<crate::contract::model::User>>
       where
           R: core::ops::RangeBounds<chrono::DateTime<chrono::Utc>> + Send;
   }
   ```

4. **`src/domain/service.rs`:**
   **Rule:** The `Service` struct encapsulates all business logic. It depends on repository traits and event publishers,
   not concrete implementations.

   ```rust
   // Example from users_info
   use super::events::UserDomainEvent;
   use super::ports::EventPublisher;

   pub struct Service {
       repo: std::sync::Arc<dyn UsersRepository>,
       events: std::sync::Arc<dyn EventPublisher<UserDomainEvent>>,
       config: ServiceConfig,
   }

   impl Service {
       pub fn new(
           repo: std::sync::Arc<dyn UsersRepository>,
           events: std::sync::Arc<dyn EventPublisher<UserDomainEvent>>,
           config: ServiceConfig,
       ) -> Self {
           Self { repo, events, config }
       }

       pub async fn create_user(&self, new_user: crate::contract::model::NewUser) -> Result<crate::contract::model::User, crate::domain::error::DomainError> {
           // Business logic here...
           let user = self.repo.insert(user.clone()).await?;

           // Publish domain event
           self.events.publish(&UserDomainEvent::Created {
               id: user.id,
               at: user.created_at,
           });

           Ok(user)
       }
   }
   ```

### Step 6: Module Wiring & Lifecycle

#### `#[modkit::module]` Full Syntax

The `#[modkit::module]` macro provides declarative registration and lifecycle management.

```rust
#[modkit::module(
    name = "my_module",
    deps = ["db"], // Dependencies on other modules
    capabilities = [db, rest, stateful],
    client = "contract::client::MyModuleApi", // Generates ClientHub helpers
    ctor = MyModule::new(), // Constructor expression (defaults to `Default`)
    lifecycle(entry = "serve", stop_timeout = "30s", await_ready) // For stateful background tasks
)]
pub struct MyModule { /* ... */ }
```

#### `ModuleCtx` Runtime Context

The `init` function receives a `ModuleCtx` struct, which provides access to essential runtime components:

- **`ctx.db()`**: Returns an `Option<Arc<db::DbHandle>>` for database access.
- **`ctx.module_config()`**: Deserializes the module's typed configuration.
- **`ctx.client_hub()`**: Provides access to the `ClientHub` for resolving other module clients.
- **`ctx.cancellation_token()`**: A `CancellationToken` for graceful shutdown of background tasks.

This is where all components are assembled and registered with ModKit.

1. **`src/module.rs` - The `#[modkit::module]` macro:**
   **Rule:** The module MUST declare `capabilities = [db, rest]`.
   **Rule:** The `client` property MUST be set to the path of your native client trait (e.g.,
   `crate::contract::client::UsersInfoApi`).
   **Checklist:** Ensure `capabilities`, `deps`, and `client` are set correctly for your module.

2. **`src/module.rs` - `impl Module for YourModule`:**
   **Rule:** The Module trait requires implementing `as_any()` method:

   ```rust
   impl Module for YourModule {
       fn as_any(&self) -> &(dyn std::any::Any + 'static) {
           self
       }

       async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
           // ... init logic
       }
   }
   ```

   **Rule:** The `init` function is the composition root. It MUST:
   1. Read the typed config: `let cfg: Config = ctx.module_config();`
   2. Get a DB handle and fail if it's missing: `let db = ctx.db().ok_or_else(...)`.
   3. Instantiate the repository, service, and any other dependencies.
   4. Store the `Arc<Service>` in a thread-safe container like `arc_swap::ArcSwapOption`.
   5. Instantiate and expose the native client to the `ClientHub` using the generated `expose_<module_name>_client`
   function.
   6. Config structs SHOULD use `#[serde(deny_unknown_fields)]` and provide safe defaults.
   7. If the module declares the `db` capability, requiring a DB handle is mandatory; fail fast when missing.

3. **`src/module.rs` - `impl DbModule` and `impl RestfulModule`:**
   **Rule:** `DbModule::migrate` MUST be implemented to run your SeaORM migrations.
   **Rule:** `RestfulModule::register_rest` MUST fail if the service is not yet initialized, then call your single
   `register_routes` function.

```rust
// Example from users_info/src/module.rs
#[modkit::module(
    name = "users_info",
    deps = ["db"],
    capabilities = [db, rest],
    client = crate::contract::client::UsersInfoApi
)]
pub struct UsersInfo {
    service: arc_swap::ArcSwapOption<Service>,
    sse: modkit::SseBroadcaster<UserEvent>, // Optional: for real-time events
}

#[async_trait::async_trait]
impl Module for UsersInfo {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        let cfg: UsersInfoConfig = ctx.module_config();
        let db = ctx.db().ok_or_else(|| anyhow::anyhow!("DB required"))?;
        let repo = SeaOrmUsersRepository::new(db.sea());

        // Create event publisher adapter for SSE (optional)
        let event_publisher: Arc<dyn EventPublisher<UserDomainEvent>> =
            Arc::new(SseUserEventPublisher::new(self.sse.clone()));

        let service = Service::new(Arc::new(repo), event_publisher, cfg.into());
        self.service.store(Some(Arc::new(service.clone())));

        let api: Arc<dyn UsersInfoApi> = Arc::new(UsersInfoLocalClient::new(Arc::new(service)));
        expose_users_info_client(ctx, &api)?;
        Ok(())
    }
    // ...
}

#[async_trait::async_trait]
impl DbModule for UsersInfo {
    async fn migrate(&self, db: &db::DbHandle) -> anyhow::Result<()> {
        crate::infra::storage::migrations::Migrator::up(db.sea(), None).await?;
        Ok(())
    }
}

impl RestfulModule for UsersInfo {
    fn register_rest(&self, _ctx: &ModuleCtx, router: axum::Router, openapi: &dyn OpenApiRegistry) -> anyhow::Result<axum::Router> {
        let service = self.service.load().as_ref().ok_or_else(|| anyhow::anyhow!("Service not initialized"))?.clone();
        let router = routes::register_routes(router, openapi, service)?;

        // Optional: Register SSE route for real-time events
        let router = routes::register_sse_route(router, openapi, self.sse.clone());

        Ok(router)
    }
}
```

#### Module Integration into the Hyperspot Binary

Your module must be integrated into the hyperspot-server binary to be loaded at runtime.

Edit `apps/hyperspot-server/Cargo.toml`:

```toml
[dependencies]
# ... existing dependencies
api_ingress = { path = "../../modules/api_ingress"}
your_module = { path = "../../modules/your-module"}  # Add this line
```

#### 2. Link module in main.rs

Edit `apps/hyperspot-server/src/main.rs` in the `_ensure_modules_linked()` function:

```rust
// Ensure modules are linked and registered via inventory
#[allow(dead_code)]
fn _ensure_modules_linked() {
    // Make sure all modules are linked
    let _ = std::any::type_name::<api_ingress::ApiIngress>();
    let _ = std::any::type_name::<your_module::YourModule>();  // Add this line
    #[cfg(feature = "users-info-example")]
    let _ = std::any::type_name::<users_info::UsersInfo>();
}
```

**Note:** Replace `your_module` with your actual module name and `YourModule` with your module struct name.

### Step 7: REST API Layer (Optional)

This layer adapts HTTP requests to domain calls. It is required only for modules exposing it's own REST API to UI or
external API clients.

#### Common principles

1. **Follow the rules below:**
   **Rule:** Strictly follow the [API guideline](./API_GUIDELINE.md).
   **Rule:** Do NOT implement a REST host. `api_ingress` owns the Axum server and OpenAPI. Modules only register routes
   via `register_routes(...)`.
   **Rule:** Use dependency injection with `Extension<Arc<Service>>` in handlers and attach the service ONCE after all
   routes are registered: `router = router.layer(Extension(service.clone()));`.
   **Rule:** Follow the `<crate>.<resource>.<action>` convention for `operation_id` naming.
   **Rule:** Always return RFC 9457 Problem Details for all 4xx/5xx errors via `ProblemResponse` (centralized in
   `api/rest/error.rs`).
   **Rule:** Observability is provided by ingress: request tracing and `X-Request-Id` are already handled. Handlers can
   access the request path (`uri.path()`) for `Problem.instance` and should use `tracing` for logs; do not set tracing
   headers manually.
   **Rule:** Do not add transport middlewares (CORS, timeouts, compression, body limits) at module level — these are
   applied globally by `api_ingress`.
   **Rule:** If you implement optimistic concurrency, honor `If-Match` with ETags and return `412 Precondition Failed`
   on mismatch. Otherwise, omit concurrency headers entirely.
   **Rule:** Do not implement rate limiting or quota headers in modules; these are applied upstream (ingress/reverse
   proxy). Avoid setting cache headers from handlers; default caching is managed at the gateway.
   **Rule:** Handlers should complete within ~30s (ingress timeout). If work may exceed that, return `202 Accepted` and
   model it as an async job (see Step 9 for SSE/long-running patterns).

2. **`src/api/rest/dto.rs`:**
   **Rule:** Create Data Transfer Objects (DTOs) for the REST API. These structs derive `serde` and `utoipa::ToSchema`.
   **Rule:** Map OpenAPI types correctly: `string: uuid` -> `uuid::Uuid`, `string: date-time` ->
   `chrono::DateTime<chrono::Utc>`.

3. **`src/api/rest/mapper.rs`:**
   **Rule:** Provide `From` implementations to convert between DTOs and `contract` models.

4. **`src/api/rest/handlers.rs`:**
   **Rule:** Handlers must be thin. They extract data, call the domain service, and map results.
   **Rule:** Use `Extension<Arc<Service>>` for dependency injection.
   **Rule:** Handler return types must match the canonical patterns:
   -   `GET` -> `Result<Json<T>, ProblemResponse>`
   -   `POST` -> `Result<(StatusCode, Json<T>), ProblemResponse>`
   -   `DELETE/PUT (no body)` -> `Result<StatusCode, ProblemResponse>`

   **Rule:** For file uploads, accept streaming bodies using `impl tokio::io::AsyncRead` (or `axum::body::Body`
   adapters) to avoid buffering the entire file in memory.

   ```rust
   // Example from users_info
   pub async fn get_user(
       Extension(svc): Extension<std::sync::Arc<Service>>,
       axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
       uri: axum::http::Uri,
   ) -> Result<axum::Json<super::dto::UserDto>, modkit::api::problem::ProblemResponse> {
       let user = svc.get_user(id).await.map_err(|e| super::error::map_domain_error(&e, uri.path()))?;
       Ok(axum::Json(user.into()))
   }
   ```

   ```rust
   // Example: file upload handler signature (streaming)
   use tokio::io::AsyncRead;
   pub async fn upload_avatar<R>(
       Extension(svc): Extension<std::sync::Arc<Service>>,
       reader: R,
   ) -> Result<axum::http::StatusCode, modkit::api::problem::ProblemResponse>
   where
       R: AsyncRead + Unpin + Send + 'static,
   {
       // stream reader to storage ...
       Ok(axum::http::StatusCode::NO_CONTENT)
   }
   ```

5. **`src/api/rest/routes.rs`:**
   **Rule:** Register ALL endpoints in a single `register_routes` function.
   **Rule:** Use `OperationBuilder` for every route, following the strict order: describe -> handler -> responses ->
   register.
   **Rule:** Register all applicable error responses, including `422 Unprocessable Entity` for validation errors when
   used.
   **Rule:** After all routes are registered, attach the service ONCE with `router.layer(Extension(service.clone()))`.

   ```rust
   // Example from users_info
   pub fn register_routes(
       mut router: axum::Router,
       openapi: &dyn modkit::api::OpenApiRegistry,
       service: std::sync::Arc<Service>,
   ) -> anyhow::Result<axum::Router> {
       // GET endpoint
       router = modkit::api::OperationBuilder::<_, _, ()>::get("/users/{id}")
           .operation_id("users_info.get_user")
           .handler(super::handlers::get_user)
           .json_response_with_schema::<super::dto::UserDto>(openapi, 200, "OK")
           .problem_response(openapi, 404, "Not Found")
           .register(router, openapi);

       // POST endpoint - CRITICAL: Use .json_request::<DTO>() not .json_request_schema()
       router = modkit::api::OperationBuilder::<_, _, ()>::post("/users")
           .operation_id("users_info.create_user")
           .summary("Create a new user")
           .description("Create a new user with the provided information")
           .tag("users")
           .json_request::<super::dto::CreateUserReq>(openapi, "User creation data")
           .handler(super::handlers::create_user)
           .json_response_with_schema::<super::dto::UserDto>(openapi, 201, "Created")
           .problem_response(openapi, 400, "Bad Request")
           .problem_response(openapi, 409, "Conflict")
           .register(router, openapi);

       // PUT endpoint
       router = modkit::api::OperationBuilder::<_, _, ()>::put("/users/{id}")
           .operation_id("users_info.update_user")
           .path_param("id", "User UUID")
           .json_request::<super::dto::UpdateUserReq>(openapi, "User update data")
           .handler(super::handlers::update_user)
           .json_response_with_schema::<super::dto::UserDto>(openapi, 200, "Updated")
           .problem_response(openapi, 400, "Bad Request")
           .problem_response(openapi, 404, "Not Found")
           .register(router, openapi);

       // DELETE endpoint
       router = modkit::api::OperationBuilder::<_, _, ()>::delete("/users/{id}")
           .operation_id("users_info.delete_user")
           .path_param("id", "User UUID")
           .handler(super::handlers::delete_user)
           .json_response(204, "User deleted successfully")
           .problem_response(openapi, 404, "Not Found")
           .register(router, openapi);

       router = router.layer(axum::Extension(service.clone()));
       Ok(router)
   }
   ```

#### OpenAPI Schema Registration for POST/PUT/DELETE

**CRITICAL:** For endpoints that accept request bodies, you MUST use `.json_request::<DTO>()` to properly register the
schema:

```rust
// CORRECT - Registers the DTO schema automatically
.json_request::< super::dto::CreateUserReq>(openapi, "Description")

// WRONG - Will cause "Invalid reference token" errors
.json_request_schema("CreateUserReq", "Description")
```

**Route Registration Patterns:**

- **GET**: `.json_response_with_schema::<ResponseDTO>()`
- **POST**: `.json_request::<RequestDTO>()` + `.json_response_with_schema::<ResponseDTO>(openapi, 201, "Created")`
- **PUT**: `.json_request::<RequestDTO>()` + `.json_response_with_schema::<ResponseDTO>(openapi, 200, "Updated")`
- **DELETE**: `.json_response(204, "Deleted")` (no request/response body typically)

### Step 8: Infra/Storage Layer (Optional)

If no database requred Skip `DbModule`, remove `db` from capabilities

This layer implements the domain's repository traits.

1. **`src/infra/storage/repositories.rs`:**
   **Rule:** Implement the repository trait using SeaORM. The implementation should be generic over `C: ConnectionTrait`
   to support both direct connections and transactions.

   ```rust
   // Example from users_info
   use sea_orm::{ConnectionTrait, EntityTrait};

   pub struct SeaOrmUsersRepository<C> where C: ConnectionTrait + Send + Sync {
       conn: C,
   }

   impl<C> SeaOrmUsersRepository<C> where C: ConnectionTrait + Send + Sync {
       pub fn new(conn: C) -> Self { Self { conn } }
   }

   #[async_trait::async_trait]
   impl<C> crate::domain::repository::UsersRepository for SeaOrmUsersRepository<C>
   where C: ConnectionTrait + Send + Sync + 'static {
       async fn find_by_id(&self, id: uuid::Uuid) -> anyhow::Result<Option<crate::contract::model::User>> {
           let found = super::entity::Entity::find_by_id(id).one(&self.conn).await?;
           Ok(found.map(Into::into))
       }
       // ... other implementations
   }
   ```

2. **`src/infra/storage/migrations/`:**
   **Rule:** Create a SeaORM migrator. This is mandatory for any module with the `db` capability.

### Step 9: SSE Integration (Optional)

If no SSE required: Remove `SseBroadcaster` and event publishing

For real-time event streaming, add Server-Sent Events support.

1. **`src/api/rest/sse_adapter.rs`:**
   **Rule:** Create an adapter that implements the domain `EventPublisher` port and forwards events to the SSE
   broadcaster.

   ```rust
   // Example from users_info
   use modkit::SseBroadcaster;
   use crate::domain::{events::UserDomainEvent, ports::EventPublisher};
   use super::dto::UserEvent;

   pub struct SseUserEventPublisher {
       out: SseBroadcaster<UserEvent>,
   }

   impl SseUserEventPublisher {
       pub fn new(out: SseBroadcaster<UserEvent>) -> Self {
           Self { out }
       }
   }

   impl EventPublisher<UserDomainEvent> for SseUserEventPublisher {
       fn publish(&self, event: &UserDomainEvent) {
           self.out.send(UserEvent::from(event));
       }
   }

   // Convert domain events to transport events
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

2. **Add SSE route registration:**
   **Rule:** Register SSE routes separately from CRUD routes, with proper timeout and Extension layers.

   ```rust
   // In api/rest/routes.rs
   pub fn register_sse_route(
       router: axum::Router,
       openapi: &dyn modkit::api::OpenApiRegistry,
       sse: modkit::SseBroadcaster<UserEvent>,
   ) -> axum::Router {
       modkit::api::OperationBuilder::<_, _, ()>::get("/users/events")
           .operation_id("users_info.events")
           .summary("User events stream (SSE)")
           .description("Real-time stream of user events as Server-Sent Events")
           .tag("users")
           .handler(handlers::users_events)
           .sse_json::<UserEvent>(openapi, "SSE stream of UserEvent")
           .register(router, openapi)
           .layer(axum::Extension(sse))
           .layer(tower_http::timeout::TimeoutLayer::new(std::time::Duration::from_secs(3600)))
   }
   ```

### Step 10: Gateway Implementation (Optional)

Implement the local client that bridges the domain service to the contract API.

1. **`src/gateways/local.rs`:**
   **Rule:** Create a local implementation of your contract client trait that delegates to the domain service.

   ```rust
   // Example from users_info
   use async_trait::async_trait;
   use std::sync::Arc;
   use crate::contract::{client::UsersInfoApi, error::UsersInfoError, model::*};
   use crate::domain::service::Service;

   pub struct UsersInfoLocalClient {
       service: Arc<Service>,
   }

   impl UsersInfoLocalClient {
       pub fn new(service: Arc<Service>) -> Self {
           Self { service }
       }
   }

   #[async_trait]
   impl UsersInfoApi for UsersInfoLocalClient {
       async fn get_user(&self, id: uuid::Uuid) -> Result<User, UsersInfoError> {
           self.service.get_user(id).await.map_err(Into::into)
       }
       // ... implement all trait methods
   }
   ```

### Step 11: Register Module in HyperSpot Server

**CRITICAL:** After creating your module, you MUST register it in the HyperSpot server application to make it discoverable and include its API endpoints in the OpenAPI documentation.

**Rule:** Every new module MUST be registered in TWO places:

1. **Add dependency in `apps/hyperspot-server/Cargo.toml`:**

   ```toml
   # user modules
   file_parser = { path = "../../modules/file_parser" }
   nodes_registry = { path = "../../modules/nodes_registry" }
   your_module = { path = "../../modules/your_module" }  # ADD THIS LINE
   ```

2. **Import module in `apps/hyperspot-server/src/registered_modules.rs`:**

   ```rust
   // This file ensures all modules are linked and registered via inventory
   #![allow(unused_imports)]

   use api_ingress as _;
   use directory_service as _;
   use file_parser as _;
   use grpc_hub as _;
   use nodes_registry as _;
   use your_module as _;  // ADD THIS LINE
   #[cfg(feature = "users-info-example")]
   use users_info as _;
   ```

**Why this is required:**
- The `inventory` crate discovers modules at link time
- Without importing the module, it won't be linked into the binary
- This results in missing API endpoints in OpenAPI documentation
- The module won't be initialized or available at runtime

**Verification:**
After registration, rebuild and run the server:
```bash
cargo build
cargo run --bin hyperspot-server -- --config config/quickstart.yaml run
```

Then check the OpenAPI documentation at `http://127.0.0.1:8087/docs` to verify your module's endpoints appear.

---

### Step 12: Testing

- **Unit Tests:** Place next to the code being tested. Mock repository traits to test domain service logic in isolation.
- **Integration/REST Tests:** Place in the `tests/` directory. Use `Router::oneshot` with a stubbed service or a real
  service connected to a test database to verify handlers, serialization, and error mapping.

- **Integration Test Template** Create `tests/integration_tests.rs` with this boilerplate:

```rust
use axum::{body::Body, http::{Request, StatusCode}, Router};
use modkit::api::{OpenApiRegistry, OperationSpec};
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;
use utoipa::openapi::Schema;

// Mock OpenAPI Registry - Required for route registration
struct MockOpenApiRegistry;

impl OpenApiRegistry for MockOpenApiRegistry {
    fn register_operation(&self, _spec: &OperationSpec) {}

    fn ensure_schema_raw(&self, name: &str, _properties: Vec<(String, utoipa::openapi::RefOr<Schema>)>) -> String {
        name.to_string()
    }

    fn as_any(&self) -> &(dyn std::any::Any + 'static) {
        self
    }
}

async fn create_test_router() -> Router {
    let service = create_test_service().await;
    let router = Router::new();
    let openapi = MockOpenApiRegistry;
    your_module::api::rest::routes::register_routes(router, &openapi, service).unwrap()
}

#[tokio::test]
async fn test_example_endpoint() {
    let router = create_test_router().await;

    let request = Request::builder()
        .uri("/your-endpoint")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
```

---

## Appendix: Operations & Quality

### A. Rust Best Practices

- **Panic Policy**: Panics mean "stop the program". Use for programming errors only, never for recoverable conditions.
  - `unwrap()` is forbidden
  - `expect()` is forbidden

- **Type Safety**:
    - All public types must be `Send` (especially futures)
    - Don't leak external crate types in public APIs
    - Use `#[expect]` for lint overrides (not `#[allow]`)

- **Initialization**: Types with 4+ initialization permutations should provide builders named `FooBuilder`.

- **Avoid Statics**: Use dependency injection instead of global statics for correctness.

- **Type Complexity**: Prefer type aliases to simplify nested generic types used widely in a module.

```rust
// Instead of complex nested types
type CapabilityStorage = Arc<RwLock<HashMap<SysCapKey, SysCapability>>>;
type DetectorStorage = Arc<RwLock<HashMap<SysCapKey, CapabilityDetector>>>;

pub struct Repository {
    capabilities: CapabilityStorage,
    detectors: DetectorStorage,
}
```

### B. Build, Quality, and Hygiene

**Rule:** Run these commands routinely during development and before commit:

```bash
# Workspace-level build and test
cargo check --workspace && cargo test --workspace

# Module-specific hygiene (replace 'your-module' with actual name)
cargo clippy --fix --lib -p your-module --allow-dirty
cargo fmt --manifest-path modules/your-module/Cargo.toml
cargo test --manifest-path modules/your-module/Cargo.toml
```

**Rule:** Clean imports (remove unused `DateTime`, test imports, trait imports).

**Rule:** Fix common issues: missing test imports (`OpenApiRegistry`, `OperationSpec`, `Schema`), type inference
errors (add explicit types), missing `chrono::Utc`, handler/service name mismatches.

**Rule:** make and CI should run: `clippy --all-targets --all-features`, `fmt --check`, `deny check`.

---

## TODO

- passing security context in all methods
- paging cursor, filtering, sorting
- events persistency
- gateways - purpose, rules, implementation
- tracing
- config
- careful review, experiment with new modules & finally sing-off
