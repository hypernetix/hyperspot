# Module Structure Patterns

**Version**: 1.0  
**Last Updated**: 2026-01-09  
**Source Analysis**: `file_parser`, `types-registry`, `analytics`

---

## Standard Module Layout

```
my-module/
├── src/
│   ├── module.rs          # Module declaration & lifecycle
│   ├── config.rs          # Configuration struct
│   ├── lib.rs             # Public exports
│   ├── api/
│   │   └── rest/
│   │       ├── mod.rs     # Re-exports
│   │       ├── routes.rs  # OperationBuilder registrations
│   │       ├── handlers.rs # Request handlers
│   │       └── dto.rs     # API data transfer objects
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── service.rs     # Business logic service
│   │   └── error.rs       # Domain errors
│   └── infra/             # Optional: DB, external APIs
│       ├── mod.rs
│       └── repository.rs
├── tests/
│   ├── integration/
│   └── common/
└── Cargo.toml
```

---

## 1. module.rs Pattern

### Standard Template

```rust
use std::sync::Arc;
use async_trait::async_trait;
use modkit::api::OpenApiRegistry;
use modkit::{Module, ModuleCtx, RestfulModule};
use tracing::{debug, info};

use crate::config::MyModuleConfig;
use crate::domain::service::MyService;

/// Module description
#[modkit::module(
    name = "my_module",
    capabilities = [rest]  // or [rest, db] or [system, rest]
)]
pub struct MyModule {
    // Use ArcSwapOption for hot-reloadable service
    service: arc_swap::ArcSwapOption<MyService>,
}

impl Default for MyModule {
    fn default() -> Self {
        Self {
            service: arc_swap::ArcSwapOption::from(None),
        }
    }
}

impl Clone for MyModule {
    fn clone(&self) -> Self {
        Self {
            service: arc_swap::ArcSwapOption::new(
                self.service.load().as_ref().map(Clone::clone)
            ),
        }
    }
}

#[async_trait]
impl Module for MyModule {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        info!("Initializing my_module");

        // 1. Load configuration
        let cfg: MyModuleConfig = ctx.config()?;
        debug!("Loaded config: {:?}", cfg);

        // 2. Create infrastructure (repos, etc.)
        // let repo = Arc::new(MyRepository::new(...));

        // 3. Create service
        let service = Arc::new(MyService::new(cfg));

        // 4. Store service for REST usage
        self.service.store(Some(service.clone()));

        // 5. Optional: Register client SDK
        // let api: Arc<dyn MyModuleApi> = Arc::new(LocalClient::new(service));
        // ctx.client_hub().register::<dyn MyModuleApi>(api);

        info!("Module initialized successfully");
        Ok(())
    }
}

impl RestfulModule for MyModule {
    fn register_rest(
        &self,
        _ctx: &ModuleCtx,
        router: axum::Router,
        openapi: &dyn OpenApiRegistry,
    ) -> anyhow::Result<axum::Router> {
        info!("Registering REST routes");

        let service = self
            .service
            .load()
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Service not initialized"))?
            .clone();

        let router = crate::api::rest::routes::register_routes(
            router,
            openapi,
            service,
        );

        info!("REST routes registered successfully");
        Ok(router)
    }
}
```

### SystemModule Pattern (optional)

For system-level modules that need post-initialization:

```rust
use modkit::contracts::SystemModule;

#[async_trait]
impl SystemModule for MyModule {
    /// Runs AFTER all modules initialized
    async fn post_init(&self, _sys: &modkit::runtime::SystemContext) -> anyhow::Result<()> {
        info!("Post-init: switching to ready mode");
        
        let service = self.service.load()
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Service not initialized"))?
            .clone();
        
        service.switch_to_ready()?;
        
        info!("Switched to ready mode successfully");
        Ok(())
    }
}
```

---

## 2. config.rs Pattern

```rust
use serde::{Deserialize, Serialize};

/// Configuration for my_module
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]  // ← Fail on unknown fields
pub struct MyModuleConfig {
    #[serde(default = "default_max_size")]
    pub max_size: u64,
    
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
    
    #[serde(default)]
    pub enabled: bool,
}

impl Default for MyModuleConfig {
    fn default() -> Self {
        Self {
            max_size: default_max_size(),
            timeout_secs: default_timeout_secs(),
            enabled: false,
        }
    }
}

fn default_max_size() -> u64 {
    100
}

fn default_timeout_secs() -> u64 {
    60
}
```

---

## 3. routes.rs Pattern

```rust
use std::sync::Arc;
use axum::{Extension, Router};
use modkit::api::{OpenApiRegistry, OperationBuilder};
use modkit::api::operation_builder::{AuthReqAction, AuthReqResource, LicenseFeature};

use super::handlers;
use super::dto::*;
use crate::domain::service::MyService;

const TAG: &str = "My Module";

// === Auth Resources ===
enum Resource {
    MyResource,
}

impl AsRef<str> for Resource {
    fn as_ref(&self) -> &'static str {
        match self {
            Resource::MyResource => "my_resource",
        }
    }
}

impl AuthReqResource for Resource {}

// === Auth Actions ===
enum Action {
    Read,
    Write,
}

impl AsRef<str> for Action {
    fn as_ref(&self) -> &'static str {
        match self {
            Action::Read => "read",
            Action::Write => "write",
        }
    }
}

impl AuthReqAction for Action {}

// === License Feature ===
struct License;

impl AsRef<str> for License {
    fn as_ref(&self) -> &'static str {
        "gts.x.core.lic.feat.v1~x.core.global.base.v1"
    }
}

impl LicenseFeature for License {}

/// Register all REST routes
#[allow(clippy::needless_pass_by_value)]
pub fn register_routes(
    mut router: Router,
    openapi: &dyn OpenApiRegistry,
    service: Arc<MyService>,
) -> Router {
    // GET /my-module/v1/items
    router = OperationBuilder::get("/my-module/v1/items")
        .operation_id("my_module.list_items")
        .summary("List items")
        .description("Retrieve all items with optional filtering")
        .tag(TAG)
        .require_auth(&Resource::MyResource, &Action::Read)
        .require_license_features::<License>([])
        .query_param("filter", false, "Filter expression")
        .handler(handlers::list_items)
        .json_response_with_schema::<ItemListDto>(
            openapi,
            http::StatusCode::OK,
            "List of items"
        )
        .standard_errors(openapi)
        .register(router, openapi);

    // GET /my-module/v1/items/{id}
    router = OperationBuilder::get("/my-module/v1/items/{id}")
        .operation_id("my_module.get_item")
        .summary("Get item by ID")
        .tag(TAG)
        .require_auth(&Resource::MyResource, &Action::Read)
        .require_license_features::<License>([])
        .path_param("id", "Item identifier")
        .handler(handlers::get_item)
        .json_response_with_schema::<ItemDto>(
            openapi,
            http::StatusCode::OK,
            "Item details"
        )
        .problem_response(openapi, http::StatusCode::NOT_FOUND, "Item not found")
        .standard_errors(openapi)
        .register(router, openapi);

    // POST /my-module/v1/items
    router = OperationBuilder::post("/my-module/v1/items")
        .operation_id("my_module.create_item")
        .summary("Create item")
        .tag(TAG)
        .require_auth(&Resource::MyResource, &Action::Write)
        .require_license_features::<License>([])
        .json_request::<CreateItemRequest>(openapi, "Item to create")
        .allow_content_types(&["application/json"])
        .handler(handlers::create_item)
        .json_response_with_schema::<ItemDto>(
            openapi,
            http::StatusCode::CREATED,
            "Item created"
        )
        .error_400(openapi)
        .standard_errors(openapi)
        .register(router, openapi);

    // Attach service via Extension
    router.layer(Extension(service))
}
```

### Public Endpoints

For endpoints that don't require authentication:

```rust
router = OperationBuilder::get("/my-module/v1/public/info")
    .operation_id("my_module.get_info")
    .summary("Get public info")
    .tag(TAG)
    .public()  // ← No authentication required
    .handler(handlers::get_info)
    .json_response_with_schema::<InfoDto>(openapi, http::StatusCode::OK, "Info")
    .standard_errors(openapi)
    .register(router, openapi);
```

---

## 4. handlers.rs Pattern

### Standard Handler

```rust
use axum::extract::{Extension, Path, Query, Json};
use modkit::api::prelude::*;
use modkit_security::SecurityCtx;
use std::sync::Arc;

use super::dto::*;
use crate::domain::service::MyService;
use crate::domain::error::DomainError;

/// GET /my-module/v1/items/{id}
#[tracing::instrument(
    skip(service, _ctx),
    fields(item_id = %id, request_id = tracing::field::Empty)
)]
#[axum::debug_handler]
pub async fn get_item(
    Path(id): Path<String>,
    Extension(_ctx): Extension<SecurityCtx>,  // ← Auto-injected by api_gateway
    Extension(service): Extension<Arc<MyService>>,
) -> ApiResult<Json<ItemDto>> {
    let item = service.get_item(&id).await
        .map_err(|e| Problem::from(e))?;
    
    Ok(Json(item.into()))
}

/// POST /my-module/v1/items
#[axum::debug_handler]
pub async fn create_item(
    Extension(ctx): Extension<SecurityCtx>,
    Extension(service): Extension<Arc<MyService>>,
    Json(req): Json<CreateItemRequest>,
) -> ApiResult<(StatusCode, Json<ItemDto>)> {
    let item = service.create_item(&ctx, req).await?;
    
    Ok((StatusCode::CREATED, Json(item.into())))
}
```

### Handler with Authz Wrapper

Alternative pattern used in `file_parser`:

```rust
use modkit_auth::axum_ext::Authz;

pub async fn get_item(
    Path(id): Path<String>,
    Authz(_ctx): Authz,  // ← Wrapper for SecurityCtx validation
    Extension(service): Extension<Arc<MyService>>,
) -> ApiResult<Json<ItemDto>> {
    // ...
}
```

---

## 5. dto.rs Pattern

```rust
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request DTO
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateItemRequest {
    pub name: String,
    pub description: Option<String>,
}

/// Response DTO
#[derive(Debug, Serialize, ToSchema)]
pub struct ItemDto {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
}

/// List response wrapper
#[derive(Debug, Serialize, ToSchema)]
pub struct ItemListDto {
    pub items: Vec<ItemDto>,
    pub count: usize,
}
```

---

## Key Patterns Summary

### Module Level
- ✅ Use `arc_swap::ArcSwapOption` for service storage
- ✅ Implement `Default` and `Clone`
- ✅ Load config via `ctx.config::<T>()?`
- ✅ Store service before registering routes
- ✅ Register client SDK if module provides one

### REST Integration
- ✅ Separate `routes.rs` from `handlers.rs`
- ✅ Use `OperationBuilder` for ALL endpoints
- ✅ Define Resource/Action enums for auth
- ✅ Attach service via `router.layer(Extension(...))`
- ✅ NEVER create `Router::new()` - extend passed router

### Handlers
- ✅ Extract `SecurityCtx` via `Extension` or `Authz`
- ✅ Extract service via `Extension<Arc<Service>>`
- ✅ Return `ApiResult<T>` or `Result<T, Problem>`
- ✅ Use `#[tracing::instrument]` for observability
- ✅ Use `#[axum::debug_handler]` during development

### Configuration
- ✅ Use `#[serde(deny_unknown_fields)]`
- ✅ Provide `Default` implementation
- ✅ Use default functions for field defaults

---

## Anti-Patterns (from real modules)

### ❌ Comments Mentioning Wrong Names

```rust
// analytics/src/module.rs:46
// ❌ BAD: Comment says api_ingress (outdated)
/// - JWT validation via api_ingress

// ✅ GOOD: Use current name
/// - JWT validation via api_gateway
```

### ❌ `.public()` Without Comment

```rust
// analytics routes
.public()  // ← Why public? Add comment

// ✅ GOOD
.public()  // TODO: Add auth after initial implementation
```

### ❌ Not Checking Service Ready State

```rust
// types-registry properly checks:
if !service.is_ready() {
    return Err(DomainError::NotInReadyMode.into());
}
```

For modules with startup validation, always check ready state in handlers.

---

## Examples from Codebase

- **Simple REST Module**: `@/modules/file_parser/src/module.rs`
- **System Module with post_init**: `@/modules/system/types-registry/types-registry/src/module.rs`
- **REST Routes**: `@/modules/file_parser/src/api/rest/routes.rs`
- **REST Handlers**: `@/modules/system/types-registry/types-registry/src/api/rest/handlers.rs`
