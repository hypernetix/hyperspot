# Module Layout and SDK Pattern

## Canonical layout (DDD-light)

Place each module under `modules/<name>/`:

```
modules/<name>/
  ├─ <name>-sdk/                     # public API surface for consumers (traits, models, errors)
  │  ├─ Cargo.toml
  │  └─ src/
  │     ├─ lib.rs
  │     ├─ client.rs|api.rs          # ClientHub trait(s)
  │     ├─ models.rs                 # transport-agnostic models (no REST specifics)
  │     └─ errors.rs                 # transport-agnostic errors
  └─ <name>/                         # module implementation
     ├─ Cargo.toml
     └─ src/
        ├─ lib.rs                    # re-exports SDK types + module struct
        ├─ module.rs                 # main struct + Module/Db/Rest/Stateful impls
        ├─ config.rs                 # typed config (optional)
        ├─ api/
        │  └─ rest/
        │     ├─ dto.rs              # HTTP DTOs (serde/utoipa) — REST-only types
        │     ├─ handlers.rs         # Axum handlers (web controllers)
        │     └─ routes.rs           # route & OpenAPI registration (OperationBuilder)
        ├─ domain/                   # internal business logic
        └─ infra/                    # “low-level”: DB, system, IO, adapters
           └─ storage/
              ├─ entity.rs           # e.g., SeaORM entities / SQL mappings
              ├─ mapper.rs           # entity <-> SDK conversions (From impls)
              └─ migrations/
                 ├─ mod.rs
                 └─ initial_001.rs
```

### Notes

- Handlers may call `domain::service` directly.
- For simple internal modules you may re-export domain models via the module crate `lib.rs`.
- Module crates host local client adapters that implement SDK traits; consumers resolve them via `ClientHub`.
- Infra uses SeaORM via the secure ORM layer (`SecureConn`) to enforce scoping. Modules cannot access raw database connections—they provide migration definitions that the runtime executes.

## SDK Crate (`<module>-sdk`)

**Purpose**: Transport-agnostic public API for consumers. Only one dependency needed.

### SDK `src/lib.rs` template

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

### SDK `src/models.rs` (transport-agnostic)

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    pub display_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewUser {
    pub id: Option<Uuid>,
    pub tenant_id: Uuid,
    pub email: String,
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UserPatch {
    pub email: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateUserRequest {
    pub id: Uuid,
    pub patch: UserPatch,
}
```

### SDK `src/errors.rs` (transport-agnostic)

```rust
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

### SDK `src/api.rs` (ClientHub trait)

```rust
use async_trait::async_trait;
use modkit_security::SecurityContext;
use uuid::Uuid;

use crate::{
    errors::UsersInfoError,
    models::{NewUser, UpdateUserRequest, User},
};
use modkit_odata::{ODataQuery, Page};

/// Public API trait for users-info module.
///
/// All methods require SecurityContext for authorization.
/// Obtain via ClientHub: `hub.get::<dyn UsersInfoClientV1>()?`
#[async_trait]
pub trait UsersInfoClientV1: Send + Sync {
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

## Module Crate (`<module>`)

### Module `src/lib.rs` template

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

### Module `src/module.rs` (registration + capabilities)

```rust
#[modkit::module(
    name = "my_module",
    deps = ["foo", "bar"], // api-gateway dependency will be added automatically for rest module capability
    capabilities = [db, rest, stateful, /* rest_host if you own the HTTP server */],
    client = my_module_sdk::MyModuleApi,
    ctor = MyModule::new(),
    lifecycle(entry = "serve", stop_timeout = "30s", await_ready)
)]
pub struct MyModule {
    /* fields */
}
```

Clients must be registered explicitly in `init()`: `ctx.client_hub().register::<dyn my_module_sdk::MyModuleApi>(api)`.

### Module `src/domain/service.rs` (domain types)

All `struct` and `enum` types in `domain/` **must** have the `#[domain_model]` attribute.
This macro enforces DDD boundaries at compile time by rejecting infrastructure types
(e.g. `sqlx`, `http`, `axum`, `reqwest`) in domain model fields.

```rust
use modkit_macros::domain_model;

#[domain_model]
pub struct Service {
    pub(super) repo: Box<dyn UserRepository>,
}

#[domain_model]
pub enum DomainError {
    NotFound { id: Uuid },
    Validation { message: String },
}
```

See also: lint [DE0309](../../dylint_lints/de03_domain_layer/de0309_must_have_domain_model/README.md) — CI enforces that no domain type is missing the attribute.

### Module `src/api/rest/dto.rs` (REST DTOs, OData)

```rust
use chrono::{DateTime, Utc};
use modkit_odata_macros::ODataFilterable;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// REST DTO for user representation with OData filtering
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, ODataFilterable)]
pub struct UserDto {
    #[odata(filter(kind = "Uuid"))]
    pub id: Uuid,
    #[odata(filter(kind = "Uuid"))]
    pub tenant_id: Uuid,
    #[odata(filter(kind = "String"))]
    pub email: String,
    pub display_name: String,
    #[odata(filter(kind = "DateTimeUtc"))]
    pub created_at: DateTime<Utc>,
    #[odata(filter(kind = "DateTimeUtc"))]
    pub updated_at: DateTime<Utc>,
}
```

### Module `src/infra/storage/entity.rs` (SeaORM)

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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

## Quick checklist

- [ ] Create `<module>-sdk` crate with `api.rs`, `models.rs`, `errors.rs`, `lib.rs`.
- [ ] Create `<module>` crate with `module.rs`, `api/rest/`, `domain/`, `infra/storage/`.
- [ ] Implement SDK trait with `async_trait` and `SecurityContext` first param.
- [ ] Add `#[domain_model]` on all `struct`/`enum` types in `domain/` (import `modkit_macros::domain_model`).
- [ ] Add `#[derive(ODataFilterable)]` on REST DTOs (import `modkit_odata_macros::ODataFilterable`).
- [ ] Add `#[derive(Scopable)]` on SeaORM entities (import `modkit_db_macros::Scopable`).
- [ ] Use `SecureConn` + `SecurityContext` for all DB operations.
- [ ] Register client in `init()`: `ctx.client_hub().register::<dyn MyModuleApi>(api)`.
- [ ] Export SDK types from module crate `lib.rs`.
