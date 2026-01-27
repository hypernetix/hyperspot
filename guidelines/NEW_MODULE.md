# New Module Guideline (Hyperspot / ModKit)

This guide provides a process for creating Hyperspot modules. It targets both human developers and LLM-based code generators.

## ModKit Core Concepts

ModKit provides a framework for building modules:

- **Composable Modules**: Discovered via `inventory` and initialized in dependency order.
- **Gateway as a Module**: `api_gateway` owns the Axum router and OpenAPI document.
- **Type-Safe REST**: An operation builder prevents half-wired routes at compile time.
- **Server-Sent Events (SSE)**: Type-safe broadcasters for real-time domain event integration.
- **Standardized HTTP Errors**: Built-in support for RFC-9457 `Problem`.
- **Typed ClientHub**: For in-process clients, resolved by interface type (with optional scope for plugins).
- **Plugin Architecture**: Scoped ClientHub + GTS-based discovery for gateway + plugins pattern.
- **Lifecycle Management**: Helpers for long-running tasks and graceful shutdown.

## HyperSpot Modular architecture

A module is a composable unit implementing typically some business logic with either REST API and/or peristent storage.
Common and stateless logic that can be reusable across modules should be implemented in the `libs` crate.

## Task Routing (for agents and focused work)

| Task / Goal | Primary section(s) to read | Related docs |
|-------------|---------------------------|--------------|
| **Create module skeleton** | Step 1 (Project layout), Step 2 (Naming matrix) | `docs/modkit_unified_system/02_module_layout_and_sdk_pattern.md` |
| **Add SDK traits/models/errors** | Step 4 (SDK crate) | `docs/modkit_unified_system/02_module_layout_and_sdk_pattern.md` |
| **Add DB entities/repositories** | Step 5 (Domain layer), Step 6 (Infra storage) | `docs/SECURE-ORM.md`, `docs/modkit_unified_system/06_secure_orm_db_access.md` |
| **Add REST endpoints** | Step 7 (REST API layer) | `docs/modkit_unified_system/04_rest_operation_builder.md` |
| **Add OData $filter/$select** | Step 7 (REST API layer, OData subsection) | `docs/ODATA_SELECT.md`, `docs/modkit_unified_system/07_odata_pagination_select_filter.md` |
| **Add errors/Problem mapping** | Step 3 (Errors management) | `docs/modkit_unified_system/05_errors_rfc9457.md` |
| **Add ClientHub inter-module calls** | Step 4 (SDK crate), Step 5 (Domain layer) | `docs/modkit_unified_system/03_clienthub_and_plugins.md` |
| **Add background tasks/lifecycle** | Step 5 (Domain layer, events) | `docs/modkit_unified_system/08_lifecycle_stateful_tasks.md` |
| **Wire module in main app** | Step 8 (Module wiring) | |
| **Write tests** | Step 7 (REST API layer, tests) | |

## Quick TOC (top-level)

- **Step 1**: Project layout (SDK + module crates)
- **Step 2**: Data types naming matrix
- **Step 3**: Errors management (Domain → SDK → Problem)
- **Step 4**: SDK crate (trait, models, errors)
- **Step 5**: Domain layer (service, events, local client)
- **Step 6**: Infra layer (config, storage, entities, repositories)
- **Step 7**: REST API layer (DTOs, handlers, routes, tests)
- **Step 8**: Module wiring (workspace, main app, registration)

## Canonical Project Layout

Modules follow a DDD-light architecture with an **SDK pattern** for public API separation:

- **`<module>-sdk`**: Separate crate containing the public API surface (trait, models, errors). Transport-agnostic.
  Consumers depend only on this.
- **`<module>`**: Module implementation crate containing domain logic, REST handlers, local client adapter, and
  infrastructure.

This SDK pattern provides:

- Clear separation between public API and implementation
- Consumers only need one lightweight dependency (`<module>-sdk`)
- Direct ClientHub registration: `hub.get::<dyn MyModuleClient>()?`

All modules MUST adhere to the following directory structure:

```
modules/<your-module>/
├─ <your-module>-sdk/           # SDK crate: public API for consumers
│  ├─ Cargo.toml
│  └─ src/
│     ├─ lib.rs                 # Re-exports: Client trait, models, errors
│     ├─ api.rs                 # Client trait (all methods take &SecurityContext)
│     ├─ models.rs              # Transport-agnostic models (NO serde)
│     └─ errors.rs              # Transport-agnostic errors
│
└─ <your-module>/               # Module implementation crate
   ├─ Cargo.toml                # Depends on <your-module>-sdk
   └─ src/
      ├─ lib.rs                 # Re-exports SDK types + module struct
      ├─ module.rs              # Module struct, #[modkit::module], trait impls
      ├─ config.rs              # Typed config with defaults
      ├─ api/                   # Transport adapters
      │  └─ rest/               # HTTP REST layer
      │     ├─ dto.rs           # DTOs (serde, ToSchema)
      │     ├─ handlers.rs      # Thin Axum handlers
      │     ├─ routes.rs        # OperationBuilder registrations
      │     ├─ error.rs         # Problem mapping (From<DomainError>)
      │     └─ sse_adapter.rs   # SSE event publisher adapter (optional)
      ├─ domain/                # Internal business logic
      │  ├─ error.rs            # Domain errors
      │  ├─ events.rs           # Domain events
      │  ├─ ports.rs            # Output ports (e.g., EventPublisher)
      │  ├─ repo.rs             # Repository traits
      │  ├─ local_client.rs     # Local client implementing SDK API trait
      │  └─ service.rs          # Service orchestrating business logic
      │      
      └─ infra/                 # Infrastructure adapters
         └─ storage/            # Database layer
            ├─ entity.rs        # SeaORM entities
            ├─ mapper.rs        # From/Into Model<->Entity
            ├─ sea_orm_repo.rs  # SeaORM repository implementation
            └─ migrations/      # SeaORM migrations
```

---

## Step-by-Step Generation Guide

> **Note:** Strictly mirror the style, naming, and structure of the `examples/modkit/users_info/` reference when
> generating
> code. This example uses the **SDK pattern** with:
> - `user_info-sdk/` — SDK crate containing the public API trait, models, and error types
> - `users_info/` — Module crate containing implementation, local client, domain, and REST handlers

### Step 1: Project & Cargo Setup

### Rules / Invariants
- **Rule**: Use SDK pattern: `<module>-sdk` for public API, `<module>` for implementation.
- **Rule**: SDK crate contains trait, models, errors; no transport specifics.
- **Rule**: Module crate implements SDK trait and registers via ClientHub.
- **Rule**: Use workspace dependencies (`{ workspace = true }`) where available.
- **Rule**: Use `time::OffsetDateTime` instead of `chrono`.
- **Rule**: Use `modkit_odata_macros::ODataFilterable` for OData filtering.
- **Rule**: Use `modkit-*` workspace dependencies, not local paths.

#### 1a. Create SDK crate `<your-module>-sdk/Cargo.toml`

**Rule:** The SDK crate contains only the public API surface with minimal dependencies.

```toml
[package]
name = "<your-module>-sdk"
version.workspace = true
edition.workspace = true
license.workspace = true
authors.workspace = true
description = "SDK for <your-module>: API trait, types, and error definitions"

[lints]
workspace = true

[dependencies]
# Core dependencies for API trait
async-trait = { workspace = true }
thiserror = { workspace = true }
uuid = { workspace = true }
time = { workspace = true }

# Security context for API methods
modkit-security = { workspace = true }

# OData support for pagination (if needed)
modkit-odata = { workspace = true }
```

#### 1b. Create module crate `<your-module>/Cargo.toml`

**Rule:** The module crate depends on the SDK and contains the full implementation.

```toml
[package]
name = "<your-module>"
version.workspace = true
publish = false
edition.workspace = true
license.workspace = true
authors.workspace = true

[lints]
workspace = true

[dependencies]
# SDK - public API, models, and errors
<your-module>-sdk = { path = "../<your-module>-sdk" }

# Core dependencies
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
time = { workspace = true, features = ["serde"] }
uuid = { workspace = true }
arc-swap = { workspace = true }
sea-orm = { workspace = true, features = ["sqlx-sqlite", "runtime-tokio-rustls", "macros", "with-time", "with-uuid"] }
sea-orm-migration = { workspace = true }
thiserror = { workspace = true }

# Local dependencies
modkit = { workspace = true }
modkit-db = { workspace = true }
modkit-db-macros = { workspace = true }
modkit-auth = { workspace = true }
modkit-security = { workspace = true }
modkit-odata = { workspace = true }
modkit-errors = { workspace = true }
modkit-errors-macro = { workspace = true }
modkit-macros = { workspace = true }

[dev-dependencies]
tower = { workspace = true, features = ["util"] }
api_gateway = { package = "cf-api-gateway", path = "../../modules/system/api_gateway" }
```

#### 1c. Create SDK `src/lib.rs`

**Rule:** The SDK lib.rs re-exports all public types for consumers.

```rust
//! <YourModule> SDK
//!
//! This crate provides the public API:
//! - `<YourModule>Client` trait for inter-module communication
//! - Model types (`User`, `NewUser`, etc.)
//! - Error type (`<YourModule>Error`)
//!
//! Consumers obtain the client from `ClientHub`:
//! ```ignore
//! let client = hub.get::<dyn YourModuleClient>()?;
//! ```

#![forbid(unsafe_code)]

pub mod api;
pub mod errors;
pub mod models;

// Re-export main types at crate root
pub use api::YourModuleClient;
pub use errors::YourModuleError;
pub use models::{NewUser, User, UserPatch, UpdateUserRequest};
```

#### 1d. Create module `src/lib.rs`

**Rule:** The module lib.rs re-exports SDK types and the module struct. Internal modules are `#[doc(hidden)]`.

```rust
//! <YourModule> Module Implementation
//!
//! The public API is defined in `<your-module>-sdk` and re-exported here.

// === PUBLIC API (from SDK) ===
pub use <your_module>_sdk::{
YourModuleClient, YourModuleError,
User, NewUser, UserPatch, UpdateUserRequest,
};

// === MODULE DEFINITION ===
pub mod module;
pub use module::YourModule;

// === INTERNAL MODULES ===
#[doc(hidden)]
pub mod api;
#[doc(hidden)]
pub mod config;
#[doc(hidden)]
pub mod domain;
#[doc(hidden)]
pub mod infra;
```

### Step 2: Data Types Naming Matrix

### Rules / Invariants
- **Rule**: Keep transport-agnostic types in the SDK crate (`<module>-sdk/src/models.rs`).
- **Rule**: SeaORM entities live in `src/infra/storage/entity.rs` with `#[derive(Scopable)]`.
- **Rule**: REST DTOs live in `src/api/rest/dto.rs` with `#[derive(ODataFilterable)]`.
- **Rule**: Use `time::OffsetDateTime` for timestamps.
- **Rule**: Conversions go in `dto.rs` or optional `mapper.rs`.

**Rule:** Use the following naming matrix for your data types:

| Operation              | DB Layer (sqlx/SeaORM)<br/>`src/infra/storage/entity.rs` | Domain Layer (SDK / domain types)<br/>`<module>-sdk/src/models.rs` | API Request (in)<br/>`src/api/rest/dto.rs`      | API Response (out)<br/>`src/api/rest/dto.rs`                                                    |
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

- Keep all transport-agnostic types in the SDK crate (e.g. `<module>-sdk/src/models.rs`). Handlers and DTOs must not leak into the SDK.
- SeaORM entities live in `src/infra/storage/entity.rs` (or submodules). Repository queries go in `src/infra/storage/sea_orm_repo.rs` or per-aggregate repo files.
- All REST DTOs (requests/responses/views) live in `src/api/rest/dto.rs`; provide `From` conversions in `dto.rs` or an optional `mapper.rs`.

### Step 3: Errors Management

### Rules / Invariants
- **Rule**: Define `DomainError` in `domain/error.rs` with `thiserror::Error`.
- **Rule**: Define SDK error in `<module>-sdk/src/errors.rs` (transport-agnostic).
- **Rule**: Implement `From<DomainError> for <Sdk>Error` in module crate.
- **Rule**: Implement `From<DomainError> for Problem` in `api/rest/error.rs`.
- **Rule**: Use `ApiResult<T>` in handlers and `?` for error propagation.
- **Rule**: Do not use `ProblemResponse` (doesn’t exist).

ModKit provides a unified error handling system with `Problem` (RFC-9457) for type-safe error
propagation.

#### Error Architecture Overview

```
DomainError (business logic)
     ↓ From impl
Problem (RFC-9457, implements IntoResponse)
     ↓
ApiResult<T> = Result<T, Problem>  (handler return type)
```

#### Errors definition

**Rule:** Use the following naming and placement matrix for error types and mappings:

| Concern                        | Type/Concept                             | File (must define)                  | Notes                                                                                                                                        |
|--------------------------------|------------------------------------------|-------------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------|
| Domain error (business)        | `DomainError`                            | `<module>/src/domain/error.rs`      | Pure business errors; no transport details. Variants reflect domain invariants (e.g., `UserNotFound`, `EmailAlreadyExists`, `InvalidEmail`). |
| SDK error (public)             | `<ModuleName>Error`                      | `<module>-sdk/src/errors.rs`        | Transport-agnostic surface for consumers. No `serde` derives. Lives in SDK crate.                                                            |
| Domain → SDK error conversion  | `impl From<DomainError> for <Sdk>Error`  | `<module>/src/domain/error.rs`      | Module crate imports SDK error and provides `From` impl.                                                                                     |
| REST error mapping             | `impl From<DomainError> for Problem`     | `<module>/src/api/rest/error.rs`    | Centralize RFC-9457 mapping via `From` trait; `Problem` implements `IntoResponse` directly.                                                  |
| Handler return type            | `ApiResult<T>`                           | `<module>/src/api/rest/handlers.rs` | Use `?` operator for error propagation; `DomainError` auto-converts to `Problem` via `From` impl.                                            |
| OpenAPI responses registration | `.error_400(openapi)`, `.error_404(...)` | `<module>/src/api/rest/routes.rs`   | Register error statuses using convenience methods on `OperationBuilder`.                                                                     |

Error design rules:

- Use situation-specific error enums grouped by concern; avoid one giant catch-all enum.
- Provide convenience `is_xxx()` helper methods on error types.
- Implement `From<DomainError> for Problem` for automatic RFC-9457 conversion.
- Provide `From<DomainError> for <Module>Error` for SDK errors.
- Use `ApiResult<T>` (which is `Result<T, Problem>`) in handlers.

#### Domain Error Template

```rust
// src/domain/error.rs
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("User not found: {id}")]
    UserNotFound { id: uuid::Uuid },

    #[error("Email already exists: {email}")]
    EmailAlreadyExists { email: String },

    #[error("Validation error on field '{field}': {message}")]
    Validation { field: String, message: String },

    #[error("Database error: {0}")]
    Database(#[from] anyhow::Error),
}

impl DomainError {
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
        }
    }
}
```

#### SDK Error (in SDK crate)

The public error type is defined in the SDK crate. See Step 4b for the full template.

#### Domain-to-SDK Error Conversion (in module crate)

**Rule:** The `From<DomainError> for <SDK>Error` impl lives in the module crate's `src/domain/error.rs`,
importing the SDK error type:

```rust
// src/domain/error.rs (in module crate)
use user_info_sdk::errors::UsersInfoError;

impl From<DomainError> for UsersInfoError {
    fn from(e: DomainError) -> Self {
        match e {
            DomainError::UserNotFound { id } => Self::not_found(id),
            DomainError::EmailAlreadyExists { email } => Self::conflict(email),
            DomainError::Validation { field, message } => {
                Self::validation(format!("{}: {}", field, message))
            }
            DomainError::Database(_) => Self::internal(),
        }
    }
}
```

#### REST Error Mapping (Problem)

**Rule:** `Problem` implements `IntoResponse` directly — no wrapper needed.

```rust
// src/api/rest/error.rs
use http::StatusCode;
use modkit::api::problem::Problem;
use crate::domain::error::DomainError;

/// Implement From<DomainError> for Problem so `?` works in handlers
impl From<DomainError> for Problem {
    fn from(e: DomainError) -> Self {
        // Extract trace ID from current tracing span if available
        let trace_id = tracing::Span::current()
            .id()
            .map(|id| id.into_u64().to_string());

        let (status, code, title, detail) = match &e {
            DomainError::UserNotFound { id } => (
                StatusCode::NOT_FOUND,
                "USERS_NOT_FOUND",
                "User not found",
                format!("No user with id {}", id),
            ),
            DomainError::EmailAlreadyExists { email } => (
                StatusCode::CONFLICT,
                "USERS_EMAIL_CONFLICT",
                "Email already exists",
                format!("Email already exists: {}", email),
            ),
            DomainError::Validation { field, message } => (
                StatusCode::BAD_REQUEST,
                "USERS_VALIDATION",
                "Bad Request",
                format!("Validation error on '{}': {}", field, message),
            ),
            DomainError::Database(_) => {
                // Log internal error, return generic message
                tracing::error!(error = ?e, "Database error occurred");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "USERS_INTERNAL",
                    "Internal Server Error",
                    "An internal error occurred".to_string(),
                )
            }
        };

        let mut problem = Problem::new(status, title, detail)
            .with_type(format!("https://errors.hyperspot.com/{}", code))
            .with_code(code);

        if let Some(id) = trace_id {
            problem = problem.with_trace_id(id);
        }

        problem
    }
}
```

#### Handler Error Pattern

**Rule:** Use `modkit::api::prelude::*` for ergonomic error handling.

```rust
// src/api/rest/handlers.rs
use modkit::api::prelude::*;
use crate::domain::error::DomainError;
use axum::{extract::Path, http::Uri, Extension};

pub async fn get_user(
    Authz(ctx): Authz,
    Extension(svc): Extension<Arc<Service>>,
    Path(id): Path<Uuid>,
) -> ApiResult<JsonBody<UserDto>> {
    // DomainError auto-converts to Problem via From impl
    let user = svc.get_user(&ctx, id).await?;
    Ok(Json(UserDto::from(user)))
}

pub async fn create_user(
    uri: Uri,
    Authz(ctx): Authz,
    Extension(svc): Extension<Arc<Service>>,
    Json(req): Json<CreateUserReq>,
) -> ApiResult<impl IntoResponse> {
    let user = svc.create_user(&ctx, req.into()).await?;
    let id_str = user.id.to_string();
    Ok(created_json(UserDto::from(user), &uri, &id_str))
}

pub async fn delete_user(
    Authz(ctx): Authz,
    Extension(svc): Extension<Arc<Service>>,
    Path(id): Path<Uuid>,
) -> ApiResult<impl IntoResponse> {
    svc.delete_user(&ctx, id).await?;
    Ok(no_content())
}
```

#### Prelude Types Reference

The `modkit::api::prelude` module provides:

| Type/Function          | Description                                               |
|------------------------|-----------------------------------------------------------|
| `ApiResult<T>`         | `Result<T, Problem>` - standard handler return type       |
| `Problem`              | RFC-9457 error type that implements `IntoResponse`        |
| `JsonBody<T>`          | Type alias for `Json<T>` response                         |
| `JsonPage<T>`          | Type alias for `Json<Page<T>>` paginated response         |
| `created_json(v, loc)` | Returns `(StatusCode::CREATED, Location header, Json(v))` |
| `no_content()`         | Returns `StatusCode::NO_CONTENT`                          |
| `Json`, `Path`         | Re-exported Axum extractors                               |

#### OpenAPI Error Registration

**Rule:** Use convenience methods instead of raw `.problem_response()`:

```rust
// src/api/rest/routes.rs
router = OperationBuilder::get("/users-info/v1/users/{id}")
    .operation_id("users_info.get_user")
    .require_auth(&Resource::Users, &Action::Read)
    .handler(handlers::get_user)
    .json_response_with_schema::<UserDto>(openapi, StatusCode::OK, "User found")
    .error_400(openapi)   // Bad Request
    .error_401(openapi)   // Unauthorized
    .error_403(openapi)   // Forbidden
    .error_404(openapi)   // Not Found
    .error_409(openapi)   // Conflict
    .error_500(openapi)   // Internal Server Error
    .register(router, openapi);
```

#### Checklist

- Implement `From<DomainError> for Problem` for automatic RFC-9457 conversion.
- Provide `From<DomainError> for <Module>Error` for SDK errors.
- Use `ApiResult<T>` (which is `Result<T, Problem>`) in handlers.
- Use `?` operator for error propagation — `DomainError` auto-converts to `Problem`.
- Use `.error_400()/.error_404()` etc. for OpenAPI registration.
- Keep all SDK errors free of `serde` and any transport specifics.
- Validation errors SHOULD use `400 Bad Request` (or `422` for structured validation).

### Step 4: SDK Crate (Public API Surface)

### Rules / Invariants
- **Rule**: SDK trait methods take `&SecurityContext` as first parameter.
- **Rule**: Use `async_trait` for the SDK trait.
- **Rule**: Models are transport-agnostic (no serde, no HTTP specifics).
- **Rule**: Errors are transport-agnostic (no serde, no Problem).
- **Rule**: Re-export all public types at crate root in `lib.rs`.

#### 4a. SDK `src/lib.rs` (re-exports)

The SDK crate (`<module>-sdk`) defines the transport-agnostic interface for your module.
Consumers depend only on this crate — not the full module implementation.

**SDK API design rules:**

- Do not expose smart pointers (`Arc<T>`, `Box<T>`) in public APIs.
- Accept `impl AsRef<str>` instead of `&str` for flexibility.
- Accept `impl AsRef<Path>` for file paths.
- Use inherent methods for core functionality; use traits for extensions.
- Public SDK types MUST implement `Debug`. Types intended for display SHOULD implement `Display`.
- **All API methods MUST accept `&SecurityContext`** as the first parameter for authorization and tenant isolation.
- **SDK types MUST NOT have `serde`** or any other transport-specific derives.

#### 4a. `<module>-sdk/src/models.rs`

**Rule:** SDK models are plain Rust structs for inter-module communication. NO `serde` derives.

```rust
// Example from user_info-sdk
use time::{OffsetDateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    pub display_name: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

/// Data for creating a new user
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewUser {
    pub id: Option<Uuid>,
    pub tenant_id: Uuid,
    pub email: String,
    pub display_name: String,
}

/// Partial update data for a user
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UserPatch {
    pub email: Option<String>,
    pub display_name: Option<String>,
}

/// Request to update a user
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateUserRequest {
    pub id: Uuid,
    pub patch: UserPatch,
}
```

#### 4b. `<module>-sdk/src/errors.rs`

**Rule:** Define a domain-specific error enum. This allows consumers to handle errors without depending on
implementation details.

```rust
// Example from user_info-sdk
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug, Clone)]
pub enum UsersInfoError {
    #[error("User not found: {id}")]
    NotFound { id: Uuid },

    #[error("User with email '{email}' already exists")]
    Conflict { email: String },

    #[error("Validation error: {message}")]
    Validation { message: String },

    #[error("Internal error")]
    Internal,
}

// Convenience constructors
impl UsersInfoError {
    pub fn not_found(id: Uuid) -> Self { Self::NotFound { id } }
    pub fn conflict(email: String) -> Self { Self::Conflict { email } }
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation { message: message.into() }
    }
    pub fn internal() -> Self { Self::Internal }
}
```

#### 4c. `<module>-sdk/src/api.rs`

**Rule:** Define the native async trait for ClientHub. Name it `<PascalCaseModule>Client`.

**Rule:** All methods MUST accept `&SecurityContext` as the first parameter.

```rust
// Example from user_info-sdk
use async_trait::async_trait;
use modkit_security::SecurityContext;
use uuid::Uuid;

use crate::{
    errors::UsersInfoError,
    models::{NewUser, UpdateUserRequest, User},
};
use modkit_odata::{ODataQuery, Page};

/// Public API trait for users_info module.
///
/// All methods require SecurityContext for authorization.
/// Obtain via ClientHub: `hub.get::<dyn UsersInfoClient>()?`
#[async_trait]
pub trait UsersInfoClient: Send + Sync {
    /// Get a user by ID
    async fn get_user(&self, ctx: &SecurityContext, id: Uuid) -> Result<User, UsersInfoError>;

    /// List users with cursor-based pagination
    async fn list_users(
        &self,
        ctx: &SecurityContext,
        query: ODataQuery,
    ) -> Result<Page<User>, UsersInfoError>;

    /// Create a new user
    async fn create_user(
        &self,
        ctx: &SecurityContext,
        new_user: NewUser,
    ) -> Result<User, UsersInfoError>;

    /// Update a user
    async fn update_user(
        &self,
        ctx: &SecurityContext,
        req: UpdateUserRequest,
    ) -> Result<User, UsersInfoError>;

    /// Delete a user by ID
    async fn delete_user(&self, ctx: &SecurityContext, id: Uuid) -> Result<(), UsersInfoError>;
}
```

**Why SecurityContext is required:**

- Enables tenant isolation (user can only access data within their tenant)
- Provides authorization context for access control checks
- Propagates user identity for audit logging
- Works seamlessly across local and gRPC transports

### Step 5: Domain Layer (Business Logic)

### Rules / Invariants
- **Rule**: Service methods take `&SecurityContext` and `&SecureConn` (or `&DbHandle`).
- **Rule**: Use `SecureConn` for all DB operations (secure-by-default).
- **Rule**: Domain events are transport-agnostic (use SDK types).
- **Rule**: Local client implements SDK trait and calls domain service.
- **Rule**: Use `CancellationToken` for background tasks.

The domain layer contains the business logic, domain events, and a local client adapter that implements the SDK trait for in-process communication.
All service methods receive `&SecurityContext` for authorization and access control.

1. **`src/domain/events.rs`:**
   **Rule:** Define transport-agnostic domain events for important business actions.

```rust
// Example from users_info
use time::OffsetDateTime;
use uuid::Uuid;
   ```rust
   // Example from users_info
   use time::OffsetDateTime;
   use uuid::Uuid;

   #[derive(Debug, Clone)]
   pub enum UserDomainEvent {
       Created { id: Uuid, at: OffsetDateTime },
       Updated { id: Uuid, at: OffsetDateTime },
       Deleted { id: Uuid, at: OffsetDateTime },
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

   **Rule:** Repository methods receive `&AccessScope` for secure data access.

   ```rust
   // Example from users_info
   use async_trait::async_trait;
   use modkit_security::AccessScope;
   use uuid::Uuid;

   // Import models from SDK crate
   use user_info_sdk::models::{NewUser, User, UserPatch};
   use modkit_odata::{ODataQuery, Page};

   #[async_trait]
   pub trait UsersRepository: Send + Sync {
       /// Find user by ID with security scoping
       async fn find_by_id(
           &self,
           scope: &AccessScope,
           id: Uuid,
       ) -> anyhow::Result<Option<User>>;

       /// Check if email exists within security scope
       async fn email_exists(
           &self,
           scope: &AccessScope,
           email: &str,
       ) -> anyhow::Result<bool>;

       /// List users with OData pagination
       async fn list_page(
           &self,
           scope: &AccessScope,
           query: ODataQuery,
       ) -> anyhow::Result<Page<User>>;

       /// Insert a new user
       async fn insert(
           &self,
           scope: &AccessScope,
           new_user: NewUser,
       ) -> anyhow::Result<User>;

       /// Update user with patch
       async fn update(
           &self,
           scope: &AccessScope,
           id: Uuid,
           patch: UserPatch,
       ) -> anyhow::Result<User>;

       /// Delete user by ID
       async fn delete(&self, scope: &AccessScope, id: Uuid) -> anyhow::Result<bool>;
   }
   ```

4. **`src/domain/service.rs`:**
   **Rule:** The `Service` struct encapsulates all business logic. It depends on repository traits and event publishers,
   not concrete implementations.

   **Rule:** All service methods accept `&SecurityContext` as the first parameter.

   ```rust
   // Example from users_info
   use std::sync::Arc;
   use modkit_security::SecurityContext;
   use uuid::Uuid;

   use super::error::DomainError;
   use super::events::UserDomainEvent;
   use super::ports::EventPublisher;
   use super::repo::UsersRepository;
   // Import models from SDK crate
   use user_info_sdk::models::{NewUser, User, UserPatch};
   use modkit_odata::{ODataQuery, Page};

   pub struct ServiceConfig {
       pub max_display_name_length: usize,
       pub default_page_size: u64,
       pub max_page_size: u64,
   }

   pub struct Service {
       repo: Arc<dyn UsersRepository>,
       events: Arc<dyn EventPublisher<UserDomainEvent>>,
       config: ServiceConfig,
   }

   impl Service {
       pub fn new(
           repo: Arc<dyn UsersRepository>,
           events: Arc<dyn EventPublisher<UserDomainEvent>>,
           config: ServiceConfig,
       ) -> Self {
           Self { repo, events, config }
       }

       pub async fn get_user(
           &self,
           ctx: &SecurityContext,
           id: Uuid,
       ) -> Result<User, DomainError> {
           self.repo
               .find_by_id(ctx, id)
               .await?
               .ok_or(DomainError::UserNotFound { id })
       }

       pub async fn list_users_page(
           &self,
           ctx: &SecurityContext,
           query: ODataQuery,
       ) -> Result<Page<User>, DomainError> {
           self.repo.list_page(ctx, query).await.map_err(Into::into)
       }

       pub async fn create_user(
           &self,
           ctx: &SecurityContext,
           new_user: NewUser,
       ) -> Result<User, DomainError> {
           // Validate email uniqueness
           if self.repo.email_exists(ctx, &new_user.email).await? {
               return Err(DomainError::EmailAlreadyExists {
                   email: new_user.email,
               });
           }

           // Insert user
           let user = self.repo.insert(ctx, new_user).await?;

           // Publish domain event
           self.events.publish(&UserDomainEvent::Created {
               id: user.id,
               at: user.created_at,
           });

           Ok(user)
       }

       pub async fn update_user(
           &self,
           ctx: &SecurityContext,
           id: Uuid,
           patch: UserPatch,
       ) -> Result<User, DomainError> {
           // Ensure user exists
           let _ = self.get_user(ctx, id).await?;

           // Update
           let user = self.repo.update(ctx, id, patch).await?;

           // Publish domain event
           self.events.publish(&UserDomainEvent::Updated {
               id: user.id,
               at: user.updated_at,
           });

           Ok(user)
       }

       pub async fn delete_user(
           &self,
           ctx: &SecurityContext,
           id: Uuid,
       ) -> Result<(), DomainError> {
           let deleted = self.repo.delete(ctx, id).await?;
           if !deleted {
               return Err(DomainError::UserNotFound { id });
           }

           // Publish domain event
           self.events.publish(&UserDomainEvent::Deleted {
               id,
               at: time::OffsetDateTime::now_utc(),
           });

           Ok(())
       }
   }
   ```

### Step 6: Module Wiring & Lifecycle

### Rules / Invariants
- **Rule**: Use `#[modkit::module(...)]` for declarative registration.
- **Rule**: Register clients explicitly in `init()` via `ctx.client_hub().register()`.
- **Rule**: Use `CancellationToken` for coordinated shutdown.
- **Rule**: Use `WithLifecycle` for background tasks.
- **Rule**: Attach service once after all routes: `router.layer(Extension(service))`.

#### `#[modkit::module]` Full Syntax

The `#[modkit::module]` macro provides declarative registration and lifecycle management.

```rust
#[modkit::module(
    name = "my_module",
    deps = ["db"], // Dependencies on other modules
    capabilities = [db, rest, stateful],
    ctor = MyModule::new(), // Constructor expression (defaults to `Default`)
    lifecycle(entry = "serve", stop_timeout = "30s", await_ready) // For stateful background tasks
)]
pub struct MyModule {
    /* ... */
}
```

> **Note:** The `client = ...` attribute is no longer used. Clients are registered **explicitly** in `init()`.

#### `ModuleCtx` Runtime Context

The `init` function receives a `ModuleCtx` struct, which provides access to essential runtime components:

| Method                     | Description                                                    |
|----------------------------|----------------------------------------------------------------|
| `ctx.config::<T>()?`       | Deserialize typed config; returns `anyhow::Result<T>`          |
| `ctx.db_required()?`       | Get DB handle or fail; returns `anyhow::Result<Arc<DbHandle>>` |
| `ctx.db()`                 | Optional DB handle; returns `Option<Arc<DbHandle>>`            |
| `ctx.client_hub()`         | Access ClientHub for registering/resolving clients             |
| `ctx.cancellation_token()` | CancellationToken for graceful shutdown                        |
| `ctx.instance_id()`        | Process-level unique instance ID (UUID)                        |

This is where all components are assembled and registered with ModKit.

1. **`src/module.rs` - The `#[modkit::module]` macro:**
   **Rule:** The module MUST declare `capabilities = [db, rest]` for REST modules with database.
   **Rule:** Do NOT use `client = ...` in the macro — register the client explicitly in `init()`.
   **Checklist:** Ensure `capabilities` and `deps` are set correctly for your module.

2. **`src/module.rs` - `impl Module for YourModule`:**

   ```rust
   impl Module for YourModule {
       async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
           // ... init logic
       }
   }
   ```

   **Rule:** The `init` function is the composition root. It MUST:
    1. Read the typed config: `let cfg: Config = ctx.config()?;`
    2. Get a DB handle: `let db = ctx.db_required()?;`
    3. Get SecureConn for security-aware queries: `let sec_conn = db.sea_secure();`
    4. Instantiate the repository, service, and any other dependencies.
    5. Store the `Arc<Service>` in a thread-safe container like `arc_swap::ArcSwapOption`.
    6. Create local client adapter and register explicitly:
       ```rust
       use <module>_sdk::api::YourModuleClient;
       let local_client = YourLocalClient::new(domain_service);
       let api: Arc<dyn YourModuleClient> = Arc::new(local_client);
       ctx.client_hub().register::<dyn YourModuleClient>(api);
       ```
    7. Config structs SHOULD use `#[serde(deny_unknown_fields)]` and provide safe defaults.

3. **`src/module.rs` - `impl DatabaseCapability` and `impl RestApiCapability`:**
   **Rule:** `DatabaseCapability::migrate` MUST be implemented to run your SeaORM migrations.
   **Rule:** `RestApiCapability::register_rest` MUST fail if the service is not yet initialized, then call your single
   `register_routes` function.

```rust
// Example from users_info/src/module.rs
use std::sync::Arc;
use async_trait::async_trait;
use modkit::api::OpenApiRegistry;
use modkit::{DatabaseCapability, Module, ModuleCtx, RestApiCapability, SseBroadcaster};
use sea_orm_migration::MigratorTrait;
use tracing::info;

use crate::api::rest::dto::UserEvent;
use crate::api::rest::routes;
use crate::api::rest::sse_adapter::SseUserEventPublisher;
use crate::config::UsersInfoConfig;
use crate::domain::events::UserDomainEvent;
use crate::domain::ports::EventPublisher;
use crate::domain::service::{Service, ServiceConfig};
use crate::infra::storage::sea_orm_repo::SeaOrmUsersRepository;

// Import API trait from SDK (not local contract module)
use user_info_sdk::api::UsersInfoClient;
// Import local client adapter
use crate::domain::local_client::UsersInfoLocalClient;

#[modkit::module(
    name = "users_info",
    capabilities = [db, rest]
    // NOTE: No `client = ...` — we register explicitly in init()
)]
pub struct UsersInfo {
    service: arc_swap::ArcSwapOption<Service>,
    sse: SseBroadcaster<UserEvent>, // Optional: for real-time events
}

impl Default for UsersInfo {
    fn default() -> Self {
        Self {
            service: arc_swap::ArcSwapOption::from(None),
            sse: SseBroadcaster::new(1024),
        }
    }
}

#[async_trait]
impl Module for UsersInfo {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        info!("Initializing users_info module");

        // Load module configuration
        let cfg: UsersInfoConfig = ctx.config()?;

        // Acquire DB with SecureConn for security-aware queries
        let db = ctx.db_required()?;
        let sec_conn = db.sea_secure();

        // Wire repository with SecureConn
        let repo = SeaOrmUsersRepository::new(sec_conn);

        // Create event publisher adapter for SSE
        let publisher: Arc<dyn EventPublisher<UserDomainEvent>> =
            Arc::new(SseUserEventPublisher::new(self.sse.clone()));

        let service_config = ServiceConfig {
            max_display_name_length: 100,
            default_page_size: cfg.default_page_size,
            max_page_size: cfg.max_page_size,
        };
        let domain_service = Arc::new(Service::new(
            Arc::new(repo),
            publisher,
            service_config,
        ));

        // Store service for REST handlers
        self.service.store(Some(domain_service.clone()));

        // === EXPLICIT CLIENT REGISTRATION ===
        // Create local client adapter that implements the SDK API trait
        let local_client = UsersInfoLocalClient::new(domain_service);
        let api: Arc<dyn UsersInfoClient> = Arc::new(local_client);

        // Register directly in ClientHub — no expose_* helper, no macro glue
        ctx.client_hub().register::<dyn UsersInfoClient>(api);
        info!("UsersInfo API registered in ClientHub via local adapter");
        Ok(())
    }
}

#[async_trait]
impl DatabaseCapability for UsersInfo {
    async fn migrate(&self, db: &modkit_db::DbHandle) -> anyhow::Result<()> {
        info!("Running users_info database migrations");
        let conn = db.sea();
        crate::infra::storage::migrations::Migrator::up(&conn, None).await?;
        Ok(())
    }
}

impl RestApiCapability for UsersInfo {
    fn register_rest(
        &self,
        _ctx: &ModuleCtx,
        router: axum::Router,
        openapi: &dyn OpenApiRegistry,
    ) -> anyhow::Result<axum::Router> {
        info!("Registering users_info REST routes");

        let service = self
            .service
            .load()
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Service not initialized"))?
            .clone();

        let router = routes::register_routes(router, openapi, service)?;

        // Optional: Register SSE route for real-time events
        let router = routes::register_users_sse_route(router, openapi, self.sse.clone());

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
api_gateway = { path = "../../modules/system/api_gateway" }
your_module = { path = "../../modules/your-module" }  # Add this line
```

#### 2. Link module in main.rs

Edit `apps/hyperspot-server/src/main.rs` in the `_ensure_modules_linked()` function:

```rust
// Ensure modules are linked and registered via inventory
#[allow(dead_code)]
fn _ensure_modules_linked() {
    // Make sure all modules are linked
    let _ = std::any::type_name::<api_gateway::ApiGateway>();
    let _ = std::any::type_name::<your_module::YourModule>();  // Add this line
    #[cfg(feature = "users-info-example")]
    let _ = std::any::type_name::<users_info::UsersInfo>();
}
```

**Note:** Replace `your_module` with your actual module name and `YourModule` with your module struct name.

### Step 7: REST API Layer (Optional)

### Rules / Invariants
- **Rule**: Use `OperationBuilder` for every route (no manual routing).
- **Rule**: Add `.require_auth()` for protected endpoints.
- **Rule**: Use `Extension<Arc<Service>>` and attach once after all routes.
- **Rule**: Use `Authz(ctx): Authz` to get `SecurityContext`.
- **Rule**: Use `ApiResult<T>` and `?` for error propagation.
- **Rule**: For OData: use `OperationBuilderODataExt` helpers and `OData(query)` extractor.
- **Rule**: Use `modkit_odata_macros::ODataFilterable` on DTOs.

This layer adapts HTTP requests to domain calls. It is required only for modules exposing their own REST API to UI or external API clients.

#### Common principles

1. **Follow the rules below:**
   **Rule:** Strictly follow the [API guideline](./DNA/REST/API.md).
   **Rule:** Do NOT implement a REST host. `api_gateway` owns the Axum server and OpenAPI. Modules only register routes
   via `register_routes(...)`.
   **Rule:** Use `Extension<Arc<Service>>` for dependency injection and attach the service ONCE after all
   routes are registered: `router = router.layer(Extension(service.clone()));`.
   **Rule:** Use `Authz(ctx): Authz` extractor for authorization — it extracts `SecurityContext` from the request.
   **Rule:** Follow the `<crate>.<resource>.<action>` convention for `operation_id` naming.
   **Rule:** Use `modkit::api::prelude::*` for ergonomic handler types (ApiResult, created_json, no_content).
   **Rule:** Always return RFC 9457 Problem Details for all 4xx/5xx errors via `Problem` (implements `IntoResponse`).
   **Rule:** Observability is provided by gateway: request tracing and `X-Request-Id` are already handled.
   **Rule:** Do not add transport middlewares (CORS, timeouts, compression, body limits) at module level.
   **Rule:** Handlers should complete within ~30s (gateway timeout). If work may exceed that, return `202 Accepted`.

2. **`src/api/rest/dto.rs`:**
   **Rule:** Create Data Transfer Objects (DTOs) for the REST API. These structs derive `serde` and `utoipa::ToSchema`.
   **Rule:** For OData filtering, add `#[derive(ODataFilterable)]` with `#[odata(filter(kind = "..."))]` on fields.
   **Rule:** Only fields annotated with `#[odata(filter(kind = "..."))]` become available for `$filter` / `$orderby` (unannotated fields are not filterable/orderable).
   **Rule:** Map OpenAPI types correctly: `string: uuid` -> `uuid::Uuid`, `string: date-time` ->
   `time::OffsetDateTime`.

   ```rust
   use time::OffsetDateTime;
   use modkit_odata_macros::ODataFilterable;
   use serde::{Deserialize, Serialize};
   use utoipa::ToSchema;
   use uuid::Uuid;

   /// REST DTO for user representation with OData filtering
   #[derive(Debug, Clone, Serialize, Deserialize, ToSchema, ODataFilterable)]
   pub struct UserDto {
       #[odata(filter(kind = "Uuid"))]
       pub id: Uuid,
       pub tenant_id: Uuid,
       #[odata(filter(kind = "String"))]
       pub email: String,
       pub display_name: String,
       #[odata(filter(kind = "OffsetDateTime"))]
       pub created_at: OffsetDateTime,
       pub updated_at: OffsetDateTime,
   }

   /// REST DTO for creating a new user
   #[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
   pub struct CreateUserReq {
       pub tenant_id: Uuid,
       pub email: String,
       pub display_name: String,
   }
   ```

3. **`src/api/rest/mapper.rs` (optional):**
   **Rule:** Provide `From` implementations to convert between DTOs and SDK models. Keep conversions near DTO definitions unless they become large.

4. **`src/api/rest/handlers.rs`:**
   **Rule:** Handlers must be thin. They extract data, call the domain service, and map results.
   **Rule:** Use `Authz(ctx): Authz` extractor to get `SecurityContext` for authorization.
   **Rule:** Use `Extension<Arc<Service>>` for dependency injection.
   **Rule:** Handler return types use the prelude helpers:

   | Pattern | Return Type | Helper |
            |---------|-------------|--------|
   | GET with body | `ApiResult<JsonBody<T>>` | `Ok(Json(dto))` |
   | POST with body | `ApiResult<impl IntoResponse>` | `Ok(created_json(dto, location))` |
   | DELETE no body | `ApiResult<impl IntoResponse>` | `Ok(no_content())` |
   | Paginated list | `ApiResult<JsonPage<T>>` | `Ok(Json(page))` |

   ```rust
   use modkit::api::prelude::*;
   use modkit::api::odata::OData;
   use modkit_auth::axum_ext::Authz;
   use crate::domain::error::DomainError;

   /// List users with cursor-based pagination
   pub async fn list_users(
       Authz(ctx): Authz,                              // Extract SecurityContext
       Extension(svc): Extension<Arc<Service>>,
       OData(query): OData,                            // OData query parameters
   ) -> ApiResult<JsonPage<UserDto>> {
       // DomainError auto-converts to Problem via From impl
       let page = svc
           .list_users_page(&ctx, query)
           .await?
           .map_items(UserDto::from);
       Ok(Json(page))
   }

   /// Get a specific user by ID
   pub async fn get_user(
       Authz(ctx): Authz,
       Extension(svc): Extension<Arc<Service>>,
       Path(id): Path<Uuid>,
   ) -> ApiResult<JsonBody<UserDto>> {
       let user = svc.get_user(&ctx, id).await?;
       Ok(Json(UserDto::from(user)))
   }

   /// Create a new user
   pub async fn create_user(
       uri: Uri,
       Authz(ctx): Authz,
       Extension(svc): Extension<Arc<Service>>,
       Json(req): Json<CreateUserReq>,
   ) -> ApiResult<impl IntoResponse> {
       let user = svc.create_user(&ctx, req.into()).await?;
       let id_str = user.id.to_string();
       Ok(created_json(UserDto::from(user), &uri, &id_str))
   }

   /// Delete a user by ID
   pub async fn delete_user(
       Authz(ctx): Authz,
       Extension(svc): Extension<Arc<Service>>,
       Path(id): Path<Uuid>,
   ) -> ApiResult<impl IntoResponse> {
       svc.delete_user(&ctx, id).await?;
       Ok(no_content())
   }
   ```

5. **`src/api/rest/routes.rs`:**
   **Rule:** Register ALL endpoints in a single `register_routes` function.
   **Rule:** Use `OperationBuilder` for every route with `.require_auth(&Resource::X, [Action::Y])` for protected endpoints.
   **Rule:** For protected endpoints, call `.require_license_features(...)` after `.require_auth(...)` (use `[]` to explicitly declare no feature requirement).
   **Rule:** For OData-enabled list endpoints, use `OperationBuilderODataExt` helpers instead of manually wiring `$filter`, `$orderby`, and `$select` via `.query_param(...)`.
   **Rule:** Use `.error_400(openapi)`, `.error_404(openapi)` etc. instead of raw `.problem_response()`.
   **Rule:** After all routes are registered, attach the service ONCE with `router.layer(Extension(service.clone()))`.

   ```rust
   use crate::api::rest::{dto, handlers};
   use crate::domain::service::Service;
   use axum::{Extension, Router};
   use modkit::api::operation_builder::{LicenseFeature, OperationBuilderODataExt};
   use modkit::api::{OpenApiRegistry, OperationBuilder};
   use std::sync::Arc;

   struct License;

   impl AsRef<str> for License {
       fn as_ref(&self) -> &'static str {
           "gts.x.core.lic.feat.v1~x.core.global.base.v1"
       }
   }

   impl LicenseFeature for License {}

   pub fn register_routes(
       mut router: Router,
       openapi: &dyn OpenApiRegistry,
       service: Arc<Service>,
   ) -> anyhow::Result<Router> {
       // GET /users - List users with cursor pagination
       router = OperationBuilder::get("/users-info/v1/users")
           .operation_id("users_info.list_users")
           .summary("List users with cursor pagination")
           .tag("users")
           .require_auth(&Resource::Users, &Action::Read)
           .require_license_features::<License>([])
           .query_param_typed("limit", false, "Max users to return", "integer")
           .query_param("cursor", false, "Cursor for pagination")
           .handler(handlers::list_users)
           .json_response_with_schema::<modkit_odata::Page<dto::UserDto>>(
               openapi,
               http::StatusCode::OK,
               "Paginated list of users",
           )
           .with_odata_filter::<dto::UserDtoFilterField>() // not .query_param("$filter", ...)
           .with_odata_select() // not .query_param("$select", ...)
           .with_odata_orderby::<dto::UserDtoFilterField>() // not .query_param("$orderby", ...)
           .error_400(openapi)
           .error_500(openapi)
           .register(router, openapi);

       // GET /users/{id} - Get a specific user
       router = OperationBuilder::get("/users-info/v1/users/{id}")
           .operation_id("users_info.get_user")
           .summary("Get user by ID")
           .tag("users")
           .require_auth(&Resource::Users, &Action::Read)
           .require_license_features::<License>([])
           .path_param("id", "User UUID")
           .handler(handlers::get_user)
           .json_response_with_schema::<dto::UserDto>(openapi, http::StatusCode::OK, "User found")
           .error_401(openapi)
           .error_403(openapi)
           .error_404(openapi)
           .error_500(openapi)
           .register(router, openapi);

       // POST /users - Create a new user
       router = OperationBuilder::post("/users-info/v1/users")
           .operation_id("users_info.create_user")
           .summary("Create a new user")
           .tag("users")
           .require_auth(&Resource::Users, &Action::Create)
           .require_license_features::<License>([])
           .json_request::<dto::CreateUserReq>(openapi, "User creation data")
           .handler(handlers::create_user)
           .json_response_with_schema::<dto::UserDto>(openapi, http::StatusCode::CREATED, "Created")
           .error_400(openapi)
           .error_401(openapi)
           .error_403(openapi)
           .error_409(openapi)
           .error_500(openapi)
           .register(router, openapi);

       // DELETE /users/{id} - Delete a user
       router = OperationBuilder::delete("/users-info/v1/users/{id}")
           .operation_id("users_info.delete_user")
           .summary("Delete user")
           .tag("users")
           .require_auth(&Resource::Users, &Action::Delete)
           .require_license_features::<License>([])
           .path_param("id", "User UUID")
           .handler(handlers::delete_user)
           .json_response(http::StatusCode::NO_CONTENT, "User deleted")
           .error_401(openapi)
           .error_403(openapi)
           .error_404(openapi)
           .error_500(openapi)
           .register(router, openapi);

       router = router.layer(Extension(service.clone()));
       Ok(router)
   }
   ```

#### OpenAPI Error Registration

**Rule:** Use convenience methods instead of raw `.problem_response()`:

| Method                      | Status Code | Description                                 |
|-----------------------------|-------------|---------------------------------------------|
| `.error_400(openapi)`       | 400         | Bad Request                                 |
| `.error_401(openapi)`       | 401         | Unauthorized                                |
| `.error_403(openapi)`       | 403         | Forbidden                                   |
| `.error_404(openapi)`       | 404         | Not Found                                   |
| `.error_409(openapi)`       | 409         | Conflict                                    |
| `.error_422(openapi)`       | 422         | Unprocessable Entity                        |
| `.error_500(openapi)`       | 500         | Internal Server Error                       |
| `.standard_errors(openapi)` | All         | Adds 400, 401, 403, 404, 409, 422, 429, 500 |

#### OpenAPI Schema Registration for POST/PUT/DELETE

**CRITICAL:** For endpoints that accept request bodies, you MUST use `.json_request::<DTO>()` to properly register the
schema:

```rust
// CORRECT - Registers the DTO schema automatically
.json_request::<dto::CreateUserReq>(openapi, "Description")

// WRONG - Will cause "Invalid reference token" errors
.json_request_schema("CreateUserReq", "Description")
```

**Route Registration Patterns:**

- **GET**: `.json_response_with_schema::<ResponseDTO>()`
- **POST**: `.json_request::<RequestDTO>()` + `.json_response_with_schema::<ResponseDTO>(openapi, 201, "Created")`
- **PUT**: `.json_request::<RequestDTO>()` + `.json_response_with_schema::<ResponseDTO>(openapi, 200, "Updated")`
- **DELETE**: `.json_response(204, "Deleted")` (no request/response body typically)

### Step 8: Infra/Storage Layer (Optional)

If no database required: skip `DatabaseCapability`, remove `db` from capabilities.

This layer implements the domain's repository traits with **Secure ORM** for tenant isolation.

> **See also:** [SECURE-ORM.md](../docs/SECURE-ORM.md) for documentation on the secure ORM layer.

#### Security Model

The Secure ORM layer provides:

- **Typestate enforcement**: Unscoped queries cannot be executed (compile-time safety)
- **Request-scoped security**: SecurityContext passed per-request from handlers
- **Tenant isolation**: Automatic WHERE clauses for multi-tenant data
- **Zero runtime overhead**: All checks happen at compile time

```
API Handler (per-request)
    ↓ Creates SecurityContext from auth
SecureConn (stateless wrapper)
    ↓ Receives AccessScope per-operation
    ↓ Applies implicit tenant/resource scope
SeaORM
    ↓
Database
```

1. **`src/infra/storage/entity.rs`:**
   **Rule:** Use `#[derive(Scopable)]` to enable secure queries on your entities.

   ```rust
   use modkit_db_macros::Scopable;
   use sea_orm::entity::prelude::*;

   #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
   #[sea_orm(table_name = "users")]
   #[secure(
       tenant_col = "tenant_id",
       resource_col = "id",
       no_owner,
       no_type
   )]
   pub struct Model {
       #[sea_orm(primary_key)]
       pub id: Uuid,
       pub tenant_id: Uuid,
       pub email: String,
       pub display_name: String,
       pub created_at: DateTimeUtc,
       pub updated_at: DateTimeUtc,
   }
   ```

2. **`src/infra/storage/sea_orm_repo.rs` (or per-aggregate repos):**
   **Rule:** Use `SecureConn` for all database operations. Pass `AccessScope` to all methods.

   ```rust
   use async_trait::async_trait;
   use modkit_db::secure::SecureConn;
   use modkit_security::AccessScope;
   use uuid::Uuid;

   // Import models from SDK crate
   use user_info_sdk::models::{NewUser, User, UserPatch};
   use crate::domain::repo::UsersRepository;
   use super::entity;
   use modkit_odata::{ODataQuery, Page};

   pub struct SeaOrmUsersRepository {
       conn: SecureConn,
   }

   impl SeaOrmUsersRepository {
       pub fn new(conn: SecureConn) -> Self {
           Self { conn }
       }
   }

   #[async_trait]
   impl UsersRepository for SeaOrmUsersRepository {
       async fn find_by_id(
           &self,
           scope: &AccessScope,
           id: Uuid,
       ) -> anyhow::Result<Option<User>> {
           // SecureConn automatically applies tenant/resource scope from AccessScope
           let found = self.conn
               .find_by_id::<entity::Entity>(scope, id)?
               .one(self.conn.conn())
               .await?;
           Ok(found.map(Into::into))
       }

       async fn list_page(
           &self,
           scope: &AccessScope,
           query: ODataQuery,
       ) -> anyhow::Result<Page<User>> {
           use modkit_db::odata::sea_orm_filter::{paginate_odata, LimitCfg};
           use crate::infra::storage::odata_mapper::UserODataMapper;
           use crate::api::rest::dto::UserDtoFilterField;

           let base_query = self.conn.find::<entity::Entity>(scope)?;

           let page = paginate_odata::<UserDtoFilterField, UserODataMapper, _, _, _, _>(
               base_query.into_inner(),
               self.conn.conn(),
               &query,
               ("id", modkit_odata::SortDir::Desc),
               LimitCfg { default: 25, max: 1000 },
               |model| model.into(),
           ).await?;

           Ok(page)
       }

       async fn insert(
           &self,
           scope: &AccessScope,
           new_user: NewUser,
       ) -> anyhow::Result<User> {
           let id = new_user.id.unwrap_or_else(uuid::Uuid::now_v7);
           let now = time::OffsetDateTime::now_utc();

           let active_model = entity::ActiveModel {
               id: sea_orm::ActiveValue::Set(id),
               tenant_id: sea_orm::ActiveValue::Set(new_user.tenant_id),
               email: sea_orm::ActiveValue::Set(new_user.email),
               display_name: sea_orm::ActiveValue::Set(new_user.display_name),
               created_at: sea_orm::ActiveValue::Set(now),
               updated_at: sea_orm::ActiveValue::Set(now),
           };

           let model = self.conn.insert::<entity::Entity>(scope, active_model).await?;
           Ok(model.into())
       }

       async fn delete(&self, scope: &AccessScope, id: Uuid) -> anyhow::Result<bool> {
           self.conn.delete_by_id::<entity::Entity>(scope, id).await
       }
   }
   ```

3. **`src/infra/storage/odata_mapper.rs`:**
   **Rule:** Implement `FieldToColumn` and `ODataFieldMapping` for OData filtering.

   ```rust
   use modkit_db::odata::sea_orm_filter::{FieldToColumn, ODataFieldMapping};
   use crate::api::rest::dto::UserDtoFilterField;
   use super::entity::{Column, Entity, Model};

   pub struct UserODataMapper;

   impl FieldToColumn<UserDtoFilterField> for UserODataMapper {
       type Column = Column;

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

4. **`src/infra/storage/migrations/`:**
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
       modkit::api::OperationBuilder::<_, _, ()>::get("/users-info/v1/users/events")
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

### Step 10: Local Client Implementation

Implement the local client adapter that bridges the domain service to the SDK API trait.
The local client implements the SDK trait and forwards calls to domain service methods.

**Location:** `src/domain/local_client.rs`. If the client implementation consists of multiple modules, create a local_client subdirectory and place all client modules there.

**Rule:** The local client:

- Implements the SDK API trait (`<module>_sdk::api::YourModuleClient`)
- Imports types from the SDK, not from a local `contract` module
- Delegates all calls to the domain `Service`
- Passes `SecurityContext` directly to service methods
- Converts `DomainError` to SDK `<Module>Error` via `From` impl

```rust
// Example: users_info/src/domain/local_client.rs
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

// Import API trait and types from SDK crate
use user_info_sdk::{
    api::UsersInfoClient,
    errors::UsersInfoError,
    models::{NewUser, UpdateUserRequest, User},
};

use crate::domain::service::Service;
use modkit_odata::{ODataQuery, Page};
use modkit_security::SecurityContext;

/// Local client adapter implementing the SDK API trait.
/// Registered in ClientHub during module init().
pub struct UsersInfoLocalClient {
    service: Arc<Service>,
}

impl UsersInfoLocalClient {
    pub fn new(service: Arc<Service>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl UsersInfoClient for UsersInfoLocalClient {
    async fn get_user(&self, ctx: &SecurityContext, id: Uuid) -> Result<User, UsersInfoError> {
        self.service
            .get_user(ctx, id)
            .await
            .map_err(Into::into)  // DomainError -> UsersInfoError via From impl
    }

    async fn list_users(
        &self,
        ctx: &SecurityContext,
        query: ODataQuery,
    ) -> Result<Page<User>, UsersInfoError> {
        self.service
            .list_users_page(ctx, query)
            .await
            .map_err(Into::into)
    }

    async fn create_user(
        &self,
        ctx: &SecurityContext,
        new_user: NewUser,
    ) -> Result<User, UsersInfoError> {
        self.service
            .create_user(ctx, new_user)
            .await
            .map_err(Into::into)
    }

    async fn update_user(
        &self,
        ctx: &SecurityContext,
        req: UpdateUserRequest,
    ) -> Result<User, UsersInfoError> {
        self.service
            .update_user(ctx, req.id, req.patch)
            .await
            .map_err(Into::into)
    }

    async fn delete_user(&self, ctx: &SecurityContext, id: Uuid) -> Result<(), UsersInfoError> {
        self.service
            .delete_user(ctx, id)
            .await
            .map_err(Into::into)
    }
}
```

**Required:** Implement `From<DomainError> for UsersInfoError` in `src/domain/error.rs`:

```rust
// src/domain/error.rs
use user_info_sdk::errors::UsersInfoError;

impl From<DomainError> for UsersInfoError {
    fn from(e: DomainError) -> Self {
        match e {
            DomainError::UserNotFound { id } => Self::not_found(id),
            DomainError::EmailAlreadyExists { email } => Self::conflict(email),
            DomainError::InvalidEmail { email } => Self::validation(format!("Invalid email: {}", email)),
            DomainError::Validation { field, message } => Self::validation(format!("{}: {}", field, message)),
            DomainError::Database { .. } => Self::internal(),
            // ... other variants
        }
    }
}
```

### Step 11: Register Module in HyperSpot Server

**CRITICAL:** After creating your module, you MUST register it in the HyperSpot server application to make it
discoverable and include its API endpoints in the OpenAPI documentation.

**Rule:** Every new module MUST be registered in TWO places:

1. **Add dependency in `apps/hyperspot-server/Cargo.toml`:**

   ```toml
   # user modules
   file_parser = { package = "cf-file-parser", path = "../../modules/file_parser" }
   nodes_registry = { package = "cf-nodes-registry", path = "../../modules/system/nodes_registry/nodes_registry" }
   your_module = { package = "cf-your-module", path = "../../modules/your_module/your_module" }  # ADD THIS LINE
   ```

2. **Import module in `apps/hyperspot-server/src/registered_modules.rs`:**

   ```rust
   // This file ensures all modules are linked and registered via inventory
   #![allow(unused_imports)]

   use api_gateway as _;
   // NOTE: built-in infrastructure modules may also be imported here in the real server,
   // but new user modules typically only need to add their own crate.
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

### Rules / Invariants
- **Rule**: Use `SecurityContext::builder()` (or `SecurityContext::anonymous()` if tenant/subject is not needed) in tests.
- **Rule**: Mock repository traits for unit tests of domain logic.
- **Rule**: Use `Router::oneshot` for integration tests with real routes.
- **Rule**: Test both success and error paths (including RFC 9457 Problem responses).
- **Rule**: Use `tokio::test` for async tests.

- **Unit Tests:** Place next to the code being tested. Mock repository traits to test domain service logic in isolation.
- **Integration/REST Tests:** Place in the `tests/` directory. Use `Router::oneshot` with a stubbed service or a real
  service connected to a test database to verify handlers, serialization, and error mapping.

#### Testing with SecurityContext

All service and repository tests need a `SecurityContext`. Use explicit tenant IDs for test contexts:

```rust
use modkit_security::SecurityContext;
use uuid::Uuid;

#[tokio::test]
async fn test_service_method() {
    let tenant_id = Uuid::new_v4();
    let subject_id = Uuid::new_v4();
    let ctx = SecurityContext::builder()
        .tenant_id(tenant_id)
        .subject_id(subject_id)
        .build();
    let service = create_test_service().await;

    let result = service.get_user(&ctx, test_user_id).await;
    assert!(result.is_ok());
}
```

For multi-tenant tests:

```rust
use modkit_security::SecurityContext;
use uuid::Uuid;

#[tokio::test]
async fn test_tenant_isolation() {
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let ctx = SecurityContext::builder()
        .tenant_id(tenant_id)
        .subject_id(user_id)
        .build();

    let service = create_test_service().await;

    // User can only see data in their tenant
    let result = service.list_users(&ctx, Default::default()).await;
    assert!(result.is_ok());
}
```

#### Integration Test Template

Create `tests/integration_tests.rs` with this boilerplate:

```rust
use axum::{body::Body, http::{Request, StatusCode}, Router};
use modkit::api::OpenApiRegistry;
use std::sync::Arc;
use tower::ServiceExt;

// Use api_gateway as the OpenAPI registry (it implements OpenApiRegistry)
use api_gateway::ApiGateway;

async fn create_test_router() -> Router {
    let service = create_test_service().await;
    let router = Router::new();
    let openapi = ApiGateway::default();
    your_module::api::rest::routes::register_routes(router, &openapi, service).unwrap()
}

#[tokio::test]
async fn test_get_endpoint() {
    let router = create_test_router().await;

    let request = Request::builder()
        .uri("/users/00000000-0000-0000-0000-000000000001")
        .header("Authorization", "Bearer test-token")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_post_endpoint() {
    let router = create_test_router().await;

    let body = serde_json::json!({
        "tenant_id": "00000000-0000-0000-0000-000000000001",
        "email": "test@example.com",
        "display_name": "Test User"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/users")
        .header("Content-Type", "application/json")
        .header("Authorization", "Bearer test-token")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}
```

#### SSE Tests

```rust
use futures::StreamExt;
use modkit::SseBroadcaster;
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_sse_broadcaster() {
    let broadcaster = SseBroadcaster::<UserEvent>::new(10);
    let mut stream = Box::pin(broadcaster.subscribe_stream());

    let event = UserEvent {
        kind: "created".to_string(),
        id: Uuid::new_v4(),
        at: Utc::now(),
    };

    broadcaster.send(event.clone());

    let received = timeout(Duration::from_millis(100), stream.next())
        .await
        .expect("timeout")
        .expect("event received");

    assert_eq!(received.kind, "created");
}
```

---

### Step 13: Out-of-Process (OoP) Module Support (Optional)

### Rules / Invariants
- **Rule**: Use SDK pattern for OoP (single `*-sdk` crate with trait, types, gRPC client).
- **Rule**: Server implementations live in module crate; SDK provides only client.
- **Rule**: Use `modkit_transport_grpc::client` utilities (`connect_with_stack`, `connect_with_retry`).
- **Rule**: Use `CancellationToken` for coordinated shutdown.
- **Rule**: Use `modkit::bootstrap::oop` for OoP bootstrap.

ModKit supports running modules as separate processes with gRPC-based inter-process communication.
This enables process isolation, language flexibility, and independent scaling.

> **See also:** [MODKIT UNIFIED SYSTEM](../docs/modkit_unified_system/README.md) for comprehensive OoP documentation.

#### When to Use OoP

- **Process isolation** — modules run in separate processes for fault isolation
- **Language flexibility** — OoP modules can be implemented in any language with gRPC support
- **Independent scaling** — modules can be scaled independently
- **Resource-intensive workloads** — separate memory/CPU limits per module

#### OoP Module Structure

OoP modules use the **contracts pattern** with three crates:

```
modules/<name>/
├── <name>-sdk/        # Shared API trait + types (NO transport)
│   ├── Cargo.toml
│   └── src/lib.rs
├── <name>-grpc/             # Proto stubs + gRPC CLIENT only
│   ├── Cargo.toml
│   ├── build.rs
│   ├── proto/<name>.proto
│   └── src/
│       ├── lib.rs
│       └── client.rs
└── <name>/                  # Module impl + gRPC SERVER + OoP binary
    ├── Cargo.toml
    └── src/
        ├── lib.rs           # Module + GrpcServiceModule impl
        └── main.rs          # OoP binary entry point
```

#### 1. SDK Crate (`<name>-sdk`)

Define the API trait and types in a separate crate (no transport dependencies):

```rust
// <name>-sdk/src/lib.rs
use async_trait::async_trait;
use modkit_security::SecurityContext;

/// Client trait for MyModule
/// All methods require SecurityContext for authorization.
#[async_trait]
pub trait MyModuleClient: Send + Sync {
    async fn do_something(
        &self,
        ctx: &SecurityContext,
        input: String,
    ) -> Result<String, MyModuleError>;
}

/// Error type for MyModule operations
#[derive(thiserror::Error, Debug)]
pub enum MyModuleError {
    #[error("gRPC transport error: {0}")]
    Transport(String),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("unauthorized: {0}")]
    Unauthorized(String),
}
```

```toml
# <name>-sdk/Cargo.toml
[package]
name = "<name>-sdk"
version.workspace = true
edition.workspace = true

[dependencies]
async-trait = { workspace = true }
modkit-security = { workspace = true }
thiserror = { workspace = true }
```

#### 2. gRPC Crate (`<name>-grpc`)

Provides proto stubs and gRPC **client only**. Server implementations are in the module crate.

```protobuf
// <name>-grpc/proto/<name>.proto
syntax = "proto3";
package mymodule.v1;

service MyModuleService {
  rpc DoSomething(DoSomethingRequest) returns (DoSomethingResponse);
}

message DoSomethingRequest {
  string input = 1;
}

message DoSomethingResponse {
  string result = 1;
}
```

```rust
// <name>-grpc/src/client.rs
use anyhow::Result;
use async_trait::async_trait;
use modkit_security::SecurityContext;
use modkit_transport_grpc::client::{connect_with_retry, GrpcClientConfig};
use modkit_transport_grpc::attach_secctx;
use tonic::transport::Channel;

use mymodule_sdk::{MyModuleClient, MyModuleError};

pub struct MyModuleGrpcClient {
    inner: crate::mymodule::my_module_service_client::MyModuleServiceClient<Channel>,
}

impl MyModuleGrpcClient {
    /// Connect with default configuration and retry logic.
    pub async fn connect(endpoint: &str) -> Result<Self> {
        let cfg = GrpcClientConfig::new("my_module");
        Self::connect_with_retry(endpoint, &cfg).await
    }

    pub async fn connect_with_retry(
        endpoint: impl Into<String>,
        cfg: &GrpcClientConfig,
    ) -> Result<Self> {
        let channel = connect_with_retry(endpoint, cfg).await?;
        Ok(Self {
            inner: crate::mymodule::my_module_service_client::MyModuleServiceClient::new(channel),
        })
    }
}

#[async_trait]
impl MyModuleClient for MyModuleGrpcClient {
    async fn do_something(
        &self,
        ctx: &SecurityContext,
        input: String,
    ) -> Result<String, MyModuleError> {
        let mut request = tonic::Request::new(crate::mymodule::DoSomethingRequest { input });
        attach_secctx(request.metadata_mut(), ctx)
            .map_err(|e| MyModuleError::Transport(e.to_string()))?;

        let response = self.inner.clone()
            .do_something(request)
            .await
            .map_err(|e| MyModuleError::Transport(e.to_string()))?;

        Ok(response.into_inner().result)
    }
}
```

```rust
// <name>-grpc/src/lib.rs
mod client;

pub mod mymodule {
    tonic::include_proto!("mymodule.v1");
}

pub use client::MyModuleGrpcClient;
pub use mymodule::my_module_service_client::MyModuleServiceClient;
pub use mymodule::my_module_service_server::{MyModuleService, MyModuleServiceServer};

pub const SERVICE_NAME: &str = "mymodule.v1.MyModuleService";
```

#### 3. Module Crate (`<name>`)

Contains local implementation, gRPC server, and OoP binary entry point.

```rust
// <name>/src/lib.rs
use std::sync::Arc;
use async_trait::async_trait;
use tonic::{Request, Response, Status};

use modkit::context::ModuleCtx;
use modkit::contracts::{GrpcServiceModule, RegisterGrpcServiceFn};
use modkit_security::SecurityContext;
use modkit_transport_grpc::extract_secctx;

// Re-export contracts and grpc for consumers
// Re-export contracts (SDK) and grpc for consumers
pub use mymodule_sdk as sdk;
pub use mymodule_grpc as grpc;

use mymodule_sdk::{MyModuleClient, MyModuleError};
use mymodule_grpc::{MyModuleService, MyModuleServiceServer, SERVICE_NAME};

/// Module struct
#[modkit::module(
    name = "my_module",
    capabilities = [grpc]
    // NOTE: No `client = ...` — we register explicitly in init()
)]
pub struct MyModule {
    api: Arc<dyn MyModuleClient>,
}

impl Default for MyModule {
    fn default() -> Self {
        Self {
            api: Arc::new(LocalImpl),
        }
    }
}

/// Local implementation of the API
struct LocalImpl;

#[async_trait]
impl MyModuleClient for LocalImpl {
    async fn do_something(
        &self,
        _ctx: &SecurityContext,
        input: String,
    ) -> Result<String, MyModuleError> {
        Ok(format!("Processed: {}", input))
    }
}

// gRPC Server Implementation
struct GrpcServer {
    api: Arc<dyn MyModuleClient>,
}

#[tonic::async_trait]
impl MyModuleService for GrpcServer {
    async fn do_something(
        &self,
        request: Request<mymodule_grpc::mymodule::DoSomethingRequest>,
    ) -> Result<Response<mymodule_grpc::mymodule::DoSomethingResponse>, Status> {
        // Extract SecurityContext from gRPC metadata
        let ctx = extract_secctx(request.metadata())?;
        let req = request.into_inner();

        let result = self.api
            .do_something(&ctx, req.input)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(mymodule_grpc::mymodule::DoSomethingResponse { result }))
    }
}

#[async_trait]
impl modkit::Module for MyModule {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        // Register local implementation in ClientHub
        ctx.client_hub().register::<dyn MyModuleClient>(self.api.clone());
        Ok(())
    }
}

#[async_trait]
impl GrpcServiceModule for MyModule {
    async fn get_grpc_services(&self, _ctx: &ModuleCtx) -> anyhow::Result<Vec<RegisterGrpcServiceFn>> {
        let server = MyModuleServiceServer::new(GrpcServer { api: self.api.clone() });

        Ok(vec![RegisterGrpcServiceFn {
            service_name: SERVICE_NAME,
            register: Box::new(move |routes| {
                routes.add_service(server.clone());
            }),
        }])
    }
}
```

```rust
// <name>/src/main.rs
use anyhow::Result;
use modkit::bootstrap::oop::{run_oop_with_options, OopRunOptions};

#[tokio::main]
async fn main() -> Result<()> {
    let opts = OopRunOptions {
        module_name: "my_module".to_string(),
        instance_id: None,  // Auto-generated UUID
        directory_endpoint: std::env::var("MODKIT_DIRECTORY_ENDPOINT")
            .unwrap_or_else(|_| "http://127.0.0.1:50051".to_string()),
        config_path: std::env::var("MODKIT_CONFIG_PATH").ok(),
        verbose: 0,
        print_config: false,
        heartbeat_interval_secs: 5,
    };

    run_oop_with_options(opts).await
}
```

#### 4. OoP Configuration

Configure OoP modules in your YAML config:

```yaml
modules:
  my_module:
    runtime:
      type: oop
      execution:
        executable_path: "~/.hyperspot/bin/my-module.exe"
        args: [ ]
        working_directory: null
        environment:
          RUST_LOG: "info"
    config:
      some_setting: "value"
```

#### 5. Wiring gRPC Client

Other modules can resolve the gRPC client via DirectoryApi:

```rust
// In consumer module's init()
use mymodule_grpc::wire_mymodule_client;

async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
    // For OoP modules, wire the gRPC client
    let directory = ctx.client_hub().get::<dyn DirectoryApi>()?;
    wire_mymodule_client(ctx.client_hub(), &*directory).await?;

    // Now the client is available
    let client = ctx.client_hub().get::<dyn MyModuleClient>()?;
    Ok(())
}
```

---

### Step 14: Plugin-Based Modules (Gateway + Plugins Pattern)

For modules that require **multiple interchangeable implementations** (e.g., different vendors, providers, or
strategies), use the **Gateway + Plugins** pattern.

> **See also:** [MODKIT_PLUGINS.md](../docs/MODKIT_PLUGINS.md) for comprehensive plugin architecture documentation.

#### When to Use Plugins

Use the plugin pattern when:

- Multiple implementations of the same interface need to coexist
- The implementation is selected at runtime based on configuration or context
- You want vendor-specific or tenant-specific behavior
- New implementations should be addable without modifying the gateway

**Examples:**

- **Authentication providers** — OAuth2, SAML, LDAP
- **Search engines** — Qdrant, Weaviate, Elasticsearch
- **LLM providers** — OpenAI, Anthropic, local models
- **File parsers** — Embedded parser, Apache Tika
- **Tenant resolvers** — Different customer backends
- **License enforcement** - Integrate with external licensing engines
- **LLM benchmarks** - Different workers
- **Persistency plugins** - Storing data to different Databases (ELK, ClickHouse)

#### Plugin Architecture Overview

```
┌────────────────────────────────────────────────────────────────────┐
│                            GATEWAY MODULE                          │
│  • Exposes public API (REST + ClientHub)                           │
│  • Selects plugin based on config/context                          │
│  • Routes calls to selected plugin                                 │
└───────────────────────────────┬────────────────────────────────────┘
                                │ hub.get_scoped::<dyn PluginClient>(&scope)
                ┌───────────────┼───────────────┐
                │               │               │
                ▼               ▼               ▼
        ┌───────────┐   ┌───────────┐   ┌───────────┐
        │ Plugin A  │   │ Plugin B  │   │ Plugin C  │
        └───────────┘   └───────────┘   └───────────┘
```

#### Crate Structure

```
modules/<gateway-name>/
├── <gateway>-sdk/              # SDK: API traits, models, errors, GTS types
│   └── src/
│       ├── api.rs              # Public API trait (gateway client)
│       ├── plugin_api.rs       # Plugin API trait (implemented by plugins)
│       ├── models.rs           # Shared models
│       ├── error.rs            # Errors
│       └── gts.rs              # GTS schema for plugin instances
│
├── <gateway>-gw/               # Gateway module
│   └── src/
│       ├── module.rs           # Module with plugin discovery
│       ├── config.rs           # Gateway config (e.g., vendor selector)
│       └── domain/
│           ├── service.rs      # Plugin resolution and delegation
│           └── local_client.rs # Public client adapter
│
└── plugins/                    # Plugin implementations
    ├── <vendor_a>_plugin/
    │   └── src/
    │       ├── module.rs       # Registers GTS instance + scoped client
    │       ├── config.rs       # Plugin config (e.g., vendor + priority)
    │       └── domain/service.rs
    └── <vendor_b>_plugin/
        └── ...
```

#### Key Implementation Steps

**1. Define Two API Traits in SDK**

**Rule:** Mirror `modules/system/tenant_resolver/tenant_resolver-sdk`:

- `src/api.rs` defines `<Module>GatewayClient`
- `src/plugin_api.rs` defines `<Module>PluginClient`

```rust
// <gateway>-sdk/src/api.rs

/// Public API — exposed by gateway, consumed by other modules
#[async_trait]
pub trait MyModuleGatewayClient: Send + Sync {
    async fn do_work(&self, ctx: &SecurityContext, input: Input) -> Result<Output, MyError>;
}

// <gateway>-sdk/src/plugin_api.rs

/// Plugin API — implemented by plugins, called by gateway
#[async_trait]
pub trait MyModulePluginClient: Send + Sync {
    async fn do_work(&self, ctx: &SecurityContext, input: Input) -> Result<Output, MyError>;
}
```

**2. Define GTS Schema for Plugin Instances**

```rust
// <gateway>-sdk/src/gts.rs
use gts_macros::struct_to_gts_schema;
use modkit::gts::BaseModkitPluginV1;

/// GTS type definition for plugin instances.
///
/// For unit struct plugins (no additional properties), use an empty unit struct.
/// The `struct_to_gts_schema` macro generates the GTS schema and helper methods.
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = BaseModkitPluginV1,
    schema_id = "gts.x.core.modkit.plugin.v1~x.y.my_module.plugin.v1~",
    description = "My Module plugin specification",
    properties = ""
)]
pub struct MyModulePluginV1;
```

**3. Gateway Registers Plugin Schema + Public Client**

The gateway is responsible for registering the plugin **schema** (GTS type definition).
Plugins only register their **instances**.

```rust
// <gateway>-gw/src/module.rs
use types_registry_sdk::TypesRegistryClient;

#[async_trait]
impl Module for MyGateway {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        let cfg: GatewayConfig = ctx.config()?;

        // === SCHEMA REGISTRATION ===
        // Gateway registers the plugin SCHEMA in types-registry.
        // Plugins only register their INSTANCES.
        // Note: types-registry is tenant-agnostic, no SecurityContext needed.
        let registry = ctx.client_hub().get::<dyn TypesRegistryClient>()?;
        let schema_str = MyModulePluginV1::gts_schema_with_refs_as_string();
        let schema_json: serde_json::Value = serde_json::from_str(&schema_str)?;
        let _ = registry
            .register(vec![schema_json])
            .await?;
        info!("Registered {} schema in types-registry",
            MyModulePluginV1::gts_schema_id().clone());

        let svc = Arc::new(Service::new(ctx.client_hub(), cfg.vendor));

        // Register PUBLIC client (no scope)
        let api: Arc<dyn MyModuleGatewayClient> = Arc::new(LocalClient::new(svc.clone()));
        ctx.client_hub().register::<dyn MyModuleGatewayClient>(api);

        self.service
            .set(svc)
            .map_err(|_| anyhow::anyhow!("Service already initialized"))?;
        Ok(())
    }
}
```

**4. Plugin Registers Instance + Scoped Client**

Each plugin registers its **instance** (metadata) and **scoped client** (implementation).
The plugin schema is already registered by the gateway.

```rust
// plugins/<vendor>_plugin/src/module.rs
use modkit::client_hub::ClientScope;
use modkit::gts::BaseModkitPluginV1;

#[async_trait]
impl Module for VendorPlugin {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        let cfg: PluginConfig = ctx.config()?;

        // Generate GTS instance ID
        let instance_id = MyModulePluginV1::gts_make_instance_id("vendor.plugins._.my_plugin.v1");

        // === INSTANCE REGISTRATION ===
        // Register the plugin INSTANCE in types-registry.
        // Note: The plugin SCHEMA is registered by the gateway module.
        // types-registry is tenant-agnostic, no SecurityContext needed.
        let registry = ctx.client_hub().get::<dyn TypesRegistryClient>()?;
        let instance = BaseModkitPluginV1::<MyModulePluginV1> {
            id: instance_id.clone(),
            vendor: cfg.vendor.clone(),
            priority: cfg.priority,
            properties: MyModulePluginV1,
        };
        let instance_json = serde_json::to_value(&instance)?;
        let _ = registry
            .register(vec![instance_json])
            .await?;

        // Create service
        let service = Arc::new(Service::new());
        self.service
            .set(service.clone())
            .map_err(|_| anyhow::anyhow!("Service already initialized"))?;

        // Register SCOPED client (with GTS instance ID as scope)
        let api: Arc<dyn MyModulePluginClient> = service;
        ctx.client_hub()
            .register_scoped::<dyn MyModulePluginClient>(ClientScope::gts_id(&instance_id), api);

        Ok(())
    }
}
```

**5. Gateway Service Resolves Plugin**

**Rule:** Mirror `modules/system/tenant_resolver/tenant_resolver-gw/src/domain/service.rs`:

- Resolve plugin instance lazily (on first use)
- Query types-registry for instances of `<Module>PluginSpecV1`
- Choose by `vendor` and lowest `priority`
- Get scoped client via `ClientScope::gts_id(instance_id)`

```rust
// <gateway>-gw/src/domain/service.rs
use std::sync::Arc;

use modkit::client_hub::{ClientHub, ClientScope};
use modkit::plugins::GtsPluginSelector;
use types_registry_sdk::{ListQuery, TypesRegistryClient};

pub struct Service {
    hub: Arc<ClientHub>,
    vendor: String,
    selector: GtsPluginSelector,
}

impl Service {
    async fn get_plugin(&self) -> Result<Arc<dyn MyModulePluginClient>, DomainError> {
        let instance_id = self.selector.get_or_init(|| self.resolve_plugin()).await?;
        let scope = ClientScope::gts_id(instance_id.as_ref());
        self.hub
            .get_scoped::<dyn MyModulePluginClient>(&scope)
            .map_err(|_| DomainError::PluginClientNotFound {
                gts_id: instance_id.to_string(),
            })
    }

    async fn resolve_plugin(&self) -> Result<String, DomainError> {
        let registry = self.hub.get::<dyn TypesRegistryClient>()?;
        let plugin_type_id = MyModulePluginSpecV1::gts_schema_id().clone();
        let instances = registry
            .list(
                ListQuery::new()
                    .with_pattern(format!("{plugin_type_id}*"))
                    .with_is_type(false),
            )
            .await?;

        choose_plugin_instance(&self.vendor, &instances)
    }
}
```

#### Module Dependencies

Ensure proper initialization order:

```rust
// Gateway depends on the types_registry and any other required modules, but not on plugins. Plugins are resolved indirectly via GTS.
#[modkit::module(
    name = "my_gateway",
    deps = ["types_registry"],
    capabilities = [rest]
)]
pub struct MyGateway { /* ... */ }

#[modkit::module(
    name = "plugin_a",
    deps = ["types_registry"],
)]
pub struct PluginA { /* ... */ }
```

#### Plugin Configuration

```yaml
# config/quickstart.yaml
modules:
  my_gateway:
    config:
      vendor: "VendorA"  # Select VendorA plugin

  plugin_a:
    config:
      vendor: "VendorA"
      priority: 10

  plugin_b:
    config:
      vendor: "VendorB"
      priority: 20
```

#### Plugin Checklist

- [ ] SDK defines both `PublicClient` trait (gateway) and `PluginClient` trait (plugins)
- [ ] SDK defines GTS schema type with `#[struct_to_gts_schema]`
- [ ] Gateway depends on `types_registry` but MUST NOT depend on plugin crates
- [ ] Gateway registers plugin **schema** using `gts_schema_with_refs_as_string()`
- [ ] Gateway registers public client WITHOUT scope
- [ ] Gateway resolves plugin lazily (after types-registry is ready)
- [ ] Each plugin depends on `types_registry`
- [ ] Each plugin registers its **instance** (not schema)
- [ ] Each plugin registers scoped client with `ClientScope::gts_id(&instance_id)`
- [ ] Plugin selection uses priority for tiebreaking

> **Note:** Use `gts_schema_with_refs_as_string()` for schema generation. This method is faster (static),
> automatically sets the correct `$id`, and generates proper `$ref` references.

#### Reference Example

For a complete reference, study the real, production-style implementation:

- `modules/system/tenant_resolver/`

It contains:

- `tenant_resolver-sdk/` — SDK with `TenantResolverGatewayClient`, `TenantResolverPluginClient`, and `TenantResolverPluginSpecV1`
- `tenant_resolver-gw/` — Gateway that registers schema and selects plugin by vendor config
- `plugins/static_tr_plugin/` — Config-based plugin (registers instance + scoped client)
- `plugins/single_tenant_tr_plugin/` — Zero-config plugin (registers instance + scoped client)

There is also a smaller example copy under:

- `examples/plugin-modules/tenant_resolver/`



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
errors (add explicit types), missing `time::OffsetDateTime`, handler/service name mismatches.

**Rule:** make and CI should run: `clippy --all-targets --all-features`, `fmt --check`, `deny check`.

---

## Further Reading

- [MODKIT UNIFIED SYSTEM](../docs/modkit_unified_system/README.md) — Complete ModKit architecture and developer guide
- [MODKIT_PLUGINS.md](../docs/MODKIT_PLUGINS.md) — Plugin architecture with Gateway + Plugins pattern
- [SECURE-ORM.md](../docs/SECURE-ORM.md) — Secure ORM layer with tenant isolation
- [TRACING_SETUP.md](../docs/TRACING_SETUP.md) — Distributed tracing with OpenTelemetry
- [DNA/REST/API.md](./DNA/REST/API.md) — REST API design principles
- [examples/modkit/users_info/](../examples/modkit/users_info/) — Reference implementation of a local module with SDK
  pattern
    - `user_info-sdk/` — SDK crate with public API trait, models, and errors
    - `users_info/` — Module implementation with local client, domain, and REST handlers
    - [examples/oop-modules/calculator_gateway/](../examples/oop-modules/calculator_gateway/) — Reference implementation of an OoP
  module 
- [examples/plugin-modules/tenant_resolver/](../examples/plugin-modules/tenant_resolver/) — Reference implementation of
  a Gateway + Plugins module
    - `tenant_resolver-sdk/` — SDK with public and plugin API traits
    - `tenant_resolver-gw/` — Gateway module with plugin discovery
    - `plugins/` — Plugin implementations (Contoso, Fabrikam)
