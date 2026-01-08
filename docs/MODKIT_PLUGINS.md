# ModKit Plugin Architecture

This guide explains how to create **plugin-based modules** in HyperSpot/ModKit. Plugins allow multiple implementations of the same interface to coexist, with the gateway selecting the appropriate plugin at runtime based on configuration or
context.

---

## Overview

ModKit supports a **Gateway + Plugin** pattern where:

- **Gateway module** — exposes a public API (REST and/or ClientHub) and routes calls to the selected plugin
- **Plugin modules** — implement a plugin API trait and register themselves for discovery
- **SDK crate** — defines both the public client API and the internal plugin API (separate traits)

This pattern enables:

- **Vendor-specific implementations** — e.g., different authentication providers, search engines, or parsers
- **Runtime selection** — choose which plugin to use based on configuration, tenant, or other context
- **Hot-pluggable extensions** — add new plugins without modifying the gateway

> [!IMPORTANT]
> **Plugin Isolation Rule:** Regular modules **cannot** depend on or consume plugin modules directly. All plugin functionality must be accessed through the Gateway Module's public API (`hub.get::<dyn GatewayClient>()`). This ensures plugin implementations remain swappable, isolated, and decoupled from consumers.

---

## Architecture Diagram

```
                            ┌─────────────────────────────┐
                            │      Other Modules          │
                            │   (consumers of gateway)    │
                            └─────────────┬───────────────┘
                                          │
                                          │ ctx.client_hub().get::<dyn PublicApi>()
                                          ▼
┌───────────────────────────────────────────────────────────────────────────────────┐
│                                   GATEWAY MODULE                                  │
│                                                                                   │
│   ┌───────────────────────────────────────────────────────────────────────────┐   │
│   │   REST API (optional)                                                     │   │
│   │   GET /my-module/v1/...                                                   │   │
│   └───────────────────────────────────────────────────────────────────────────┘   │
│                                         │                                         │
│                                         │ calls domain service                    │
│                                         ▼                                         │
│   ┌───────────────────────────────────────────────────────────────────────────┐   │
│   │   Domain Service                                                          │   │
│   │   - Queries types-registry for plugin instances                           │   │
│   │   - Selects plugin based on context or config (vendor, priority, etc.)    │   │
│   │   - Resolves plugin client from ClientHub (scoped)                        │   │
│   └───────────────────────────────────────────────────────────────────────────┘   │
│                                         │                                         │
│                                         │ hub.get_scoped::<dyn PluginClient>(&scope)
│                                         ▼                                         │
└───────────────────────────────────────────────────────────────────────────────────┘
                                          │
          ┌───────────────────────────────┼────────────────────────────────┐
          │                               │                                │
          ▼                               ▼                                ▼
┌───────────────────┐           ┌───────────────────┐           ┌───────────────────┐
│  PLUGIN A         │           │  PLUGIN B         │           │  PLUGIN C         │
│  (contoso_impl)   │           │  (fabrikam_impl)  │           │  (custom_impl)    │
│                   │           │                   │           │                   │
│  Implements:      │           │  Implements:      │           │  Implements:      │
│  dyn PluginClient │           │  dyn PluginClient │           │  dyn PluginClient │
│                   │           │                   │           │                   │
│  Registers:       │           │  Registers:       │           │  Registers:       │
│  - GTS instance   │           │  - GTS instance   │           │  - GTS instance   │
│  - Scoped client  │           │  - Scoped client  │           │  - Scoped client  │
└───────────────────┘           └───────────────────┘           └───────────────────┘
```

---

## Key Concepts

### 1. Two API Traits (Public vs Plugin)

The SDK defines **two separate traits**:

```rust
/// Public API — exposed by the gateway to other modules
/// Registered WITHOUT a scope in ClientHub
#[async_trait]
pub trait MyModuleGatewayClient: Send + Sync {
    async fn do_something(&self, ctx: &SecurityCtx, input: Input) -> Result<Output, MyError>;
}

/// Plugin API — implemented by plugins, called by the gateway
/// Registered WITH a scope (GTS instance ID) in ClientHub
#[async_trait]
pub trait MyModulePluginClient: Send + Sync {
    async fn do_something(&self, ctx: &SecurityCtx, input: Input) -> Result<Output, MyError>;
}
```

**Why two traits?**

- The public trait is the stable contract for consumers — they don't know or care which plugin is used
- The plugin trait may have different method signatures or additional methods only the gateway uses
- Consumers call `hub.get::<dyn MyModuleGatewayClient>()` — gateway handles plugin routing internally

### 2. GTS Instance IDs for Plugin Discovery

Each plugin instance is identified by a **GTS (Global Type System) ID**:

```
gts.x.core.modkit.plugin.v1~<vendor>.<package_name>.<module_name>.plugin.v1~
└─────────────────────────┘ └──────────────────────────────────────────────┘
    Base plugin type ID           Specific module plugin interface ID
```

**Note:** The base plugin type `gts.x.core.modkit.plugin.v1~` is automatically registered by
the `types_registry` module during initialization. You don't need to register it manually.

Example instance IDs:

- `gts.x.core.modkit.plugin.v1~x.core.tenant_resolver.plugin.v1~contoso.app._.plugin.v1`
- `gts.x.core.modkit.plugin.v1~x.core.tenant_resolver.plugin.v1~fabrikam.app._.plugin.v1`

GTS provides:

- **Stable, versioned identifiers** for both schemas and instances
- **Schema-driven validation** of instance content
- **Registry-based discovery** of available plugins (e.g. `gts.x.core.modkit.plugin.v1~x.core.tenant_resolver.plugin.v1~*`)

### 3. Scoped Clients in ClientHub

The `ClientHub` supports **scoped clients** for plugin-like scenarios:

```rust
// Plugin registers its implementation with a scope
let scope = ClientScope::gts_id(&instance_id);
ctx.client_hub().register_scoped::<dyn MyModulePluginClient>(scope, plugin_impl);

// Gateway resolves the selected plugin's client
let scope = ClientScope::gts_id(&selected_instance_id);
let plugin = hub.get_scoped::<dyn MyModulePluginClient>(&scope)?;
```

This allows multiple implementations of the same trait to coexist, each keyed by its GTS instance ID.

### 4. types-registry for Runtime Discovery

The `types-registry` module provides:

- **Schema registration** — register GTS schemas for validation
- **Instance registration** — register plugin instances with validated content
- **Discovery queries** — list instances matching a pattern

**Registration responsibility:**

| What | Who registers | When |
|------|--------------|------|
| Core GTS types (e.g., `gts.x.core.modkit.plugin.v1~`) | **types_registry module** | Automatically during module init |
| Plugin **schema** (GTS type definition) | **Gateway module** | During gateway `init()` |
| Plugin **instance** (specific implementation) | **Each plugin** | During plugin `init()` |

This separation ensures:
- Core framework types are always available for all modules
- Schema is registered once by the authoritative owner (the gateway)
- Plugins only declare their own existence via instance registration
- Clear ownership and simpler plugin implementations

**Gateway registers schema:**

```rust
// In gateway module init()
let registry = ctx.client_hub().get::<dyn TypesRegistryApi>()?;

// Register schema using GTS-provided method for proper $id and $ref handling
let schema_str = MyModulePluginSpecV1::gts_schema_with_refs_as_string();
let schema_json: serde_json::Value = serde_json::from_str(&schema_str)?;
registry.register(&SecurityCtx::root_ctx(), vec![schema_json]).await?;
```

**Plugin registers instance:**

```rust
// In plugin module init()
let registry = ctx.client_hub().get::<dyn TypesRegistryApi>()?;

// Register instance only (schema is already registered by gateway)
let instance = BaseModkitPluginV1::<MyModulePluginSpecV1> {
    id: instance_id.clone(),
    vendor: "Contoso".into(),
    priority: 10,
    properties: MyModulePluginSpecV1,
};
let instance_json = serde_json::to_value(&instance)?;
let _ = registry
    .register(&SecurityCtx::root_ctx(), vec![instance_json])
    .await?;
```

> **Note:** Use `gts_schema_with_refs_as_string()` for schema generation. This method is faster (static),
> automatically sets the correct `$id`, and generates proper `$ref` references.

---

## Crate Structure

A plugin-based module has this structure:

```
modules/<gateway-name>/
├── <gateway>-sdk/              # SDK crate: API traits, models, errors, GTS types
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs              # Re-exports: PublicClient, PluginClient, models, errors
│       ├── api.rs              # Both trait definitions (PublicClient + PluginClient)
│       ├── models.rs           # Shared models for both APIs
│       ├── error.rs            # Transport-agnostic errors
│       └── gts.rs              # GTS schema types for plugin instances
│
├── <gateway>-gw/               # Gateway module implementation
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs              # Re-exports SDK + module struct
│       ├── module.rs           # Module declaration, init, REST registration
│       ├── config.rs           # Gateway config (e.g., vendor selection)
│       ├── local_client.rs     # Public client adapter (implements PublicClient)
│       ├── api/rest/           # REST handlers, DTOs, routes
│       └── domain/
│           ├── service.rs      # Plugin resolution and delegation
│           └── error.rs        # Domain errors
│
└── plugins/                    # Plugin implementations
    ├── <vendor_a>_plugin/
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs          # Module exports
    │       ├── module.rs       # Module declaration with types-registry + scoped client registration
    │       ├── config.rs       # Plugin config (vendor, priority)
    │       └── domain/
    │           └── service.rs  # Plugin implementation (implements PluginClient)
    │
    └── <vendor_b>_plugin/
        └── ...                 # Same structure
```

---

## Step-by-Step: Creating a Plugin System

### Step 1: Define the SDK

Create `<gateway>-sdk/` with both API traits:

```rust
// <gateway>-sdk/src/api.rs

use async_trait::async_trait;
use modkit_security::SecurityCtx;

/// Public API for consumers (registered without scope by gateway)
#[async_trait]
pub trait MyModuleGatewayClient: Send + Sync {
    async fn get_data(&self, ctx: &SecurityCtx, id: &str) -> Result<Data, MyError>;
    async fn list_data(&self, ctx: &SecurityCtx, query: Query) -> Result<Page<Data>, MyError>;
}

/// Plugin API (registered with scope by each plugin)
#[async_trait]
pub trait MyModulePluginClient: Send + Sync {
    async fn get_data(&self, ctx: &SecurityCtx, id: &str) -> Result<Data, MyError>;
    async fn list_data(&self, ctx: &SecurityCtx, query: Query) -> Result<Page<Data>, MyError>;
}
```

Define the GTS schema for plugin instances:

```rust
// <gateway>-sdk/src/gts.rs

use gts_macros::struct_to_gts_schema;
use modkit::gts::BaseModkitPluginV1;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// GTS type definition for plugin instances.
///
/// For unit struct plugins (no additional properties), use an empty unit struct.
/// The `struct_to_gts_schema` macro generates the GTS schema and helper methods.
///
/// GTS ID format: `gts.x.core.modkit.plugin.v1~<vendor>.<package>.<module>.plugin.v1~`
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = BaseModkitPluginV1,
    schema_id = "gts.x.core.modkit.plugin.v1~vendor.pkg.my_module.plugin.v1~",
    description = "My Module plugin specification",
    properties = ""
)]
pub struct MyModulePluginSpecV1;
```

### Step 2: Implement the Gateway

The gateway module:

1. Registers the plugin **schema** in types-registry (once, for all plugins)
2. Loads configuration (e.g., which vendor to use)
3. Queries types-registry for plugin instances
4. Selects the best plugin based on criteria
5. Registers a public client in ClientHub

```rust
// <gateway>-gw/src/module.rs

use std::sync::Arc;
use async_trait::async_trait;
use modkit::{Module, ModuleCtx};
use modkit_security::SecurityCtx;
use my_sdk::{MyModuleGatewayClient, MyModulePluginSpecV1};
use types_registry_sdk::TypesRegistryApi;

#[modkit::module(
    name = "my_gateway",
    deps = ["types_registry", "plugin_a", "plugin_b"],  // Depend on all plugins
    capabilities = [rest]
)]
pub struct MyGateway {
    service: arc_swap::ArcSwapOption<Service>,
}

#[async_trait]
impl Module for MyGateway {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        let cfg: GatewayConfig = ctx.config()?;

        // === SCHEMA REGISTRATION ===
        // Gateway is responsible for registering the plugin SCHEMA.
        // Plugins only register their INSTANCES.
        let registry = ctx.client_hub().get::<dyn TypesRegistryApi>()?;
        let schema_str = MyModulePluginSpecV1::gts_schema_with_refs_as_string();
        let schema_json: serde_json::Value = serde_json::from_str(&schema_str)?;
        let _ = registry
            .register(&SecurityCtx::root_ctx(), vec![schema_json])
            .await?;
        info!("Registered {} schema in types-registry",
            MyModulePluginSpecV1::gts_schema_id().clone());

        // Create service with lazy plugin resolution
        let svc = Arc::new(Service::new(ctx.client_hub(), cfg.vendor));

        // Register PUBLIC client (no scope) for other modules
        let api: Arc<dyn MyModuleGatewayClient> = Arc::new(LocalClient::new(svc.clone()));
        ctx.client_hub().register::<dyn MyModuleGatewayClient>(api);

        self.service.store(Some(svc));
        Ok(())
    }
}
```

#### Gateway REST requirements (access control, licensing, OData)

When the gateway exposes REST endpoints, route definitions follow the same ModKit conventions as regular modules:

- **Access control**: use `.require_auth(&Resource::X, &Action::Y)` for protected operations.
- **License check**: for authenticated operations, calling `.require_license_features::<F>(...)` is mandatory (use `[]` to explicitly declare no license feature requirement).
- **OData query options**: for list endpoints, use `OperationBuilderODataExt` helpers instead of manually registering `$filter`, `$orderby`, and `$select` query params.
- **OData DTO annotations**: list DTOs must derive `ODataFilterable`, and each filterable/orderable field must be annotated with `#[odata(filter(kind = "..."))]` to generate the `*FilterField` enum used by `.with_odata_filter::<...>()` and `.with_odata_orderby::<...>()`.

Example (gateway `routes.rs`):

```rust
use modkit::api::operation_builder::{LicenseFeature, OperationBuilderODataExt};
use modkit::api::{OpenApiRegistry, OperationBuilder};

router = OperationBuilder::get("/my-gateway/v1/items")
    .operation_id("my_gateway.list_items")
    .require_auth(&Resource::Items, &Action::Read)
    .require_license_features::<License>([])
    .with_odata_filter::<dto::ItemDtoFilterField>()
    .with_odata_select()
    .with_odata_orderby::<dto::ItemDtoFilterField>()
    .handler(handlers::list_items)
    .json_response_with_schema::<modkit_odata::Page<dto::ItemDto>>(
        openapi,
        http::StatusCode::OK,
        "Paginated list of items",
    )
    .register(router, openapi);
```

The domain service handles plugin resolution:

```rust
// <gateway>-gw/src/domain/service.rs

use modkit::client_hub::{ClientHub, ClientScope};
use my_sdk::{MyModulePluginClient, MyModulePluginSpec};
use tokio::sync::OnceCell;
use types_registry_sdk::TypesRegistryApi;

pub struct Service {
    hub: Arc<ClientHub>,
    vendor: String,
    resolved: OnceCell<ClientScope>,  // Cache the resolved plugin scope
}

impl Service {
    /// Lazily resolve the plugin on first call
    async fn get_plugin(&self) -> Result<Arc<dyn MyMoodulePluginClient>, DomainError> {
        let scope = self.resolved
            .get_or_try_init(|| self.resolve_plugin())
            .await?;

        self.hub
            .get_scoped::<dyn MyModulePluginClient>(scope)
            .map_err(|_| DomainError::PluginClientNotFound)
    }

    async fn resolve_plugin(&self) -> Result<ClientScope, DomainError> {
        let registry = self.hub.get::<dyn TypesRegistryApi>()?;

        // Query for plugin instances
        let plugin_type_id = MyModulePluginSpecV1::gts_schema_id().clone();
        let instances = registry
            .list(
                &SecurityCtx::root_ctx(),
                ListQuery::new()
                    .with_pattern(format!("{}*", plugin_type_id))
                    .with_is_type(false),
            )
            .await?;

        // Select best plugin based on vendor and priority
        let selected = choose_plugin(&self.vendor, &instances)?;
        Ok(ClientScope::gts_id(&selected.gts_id))
    }

    pub async fn get_data(&self, ctx: &SecurityCtx, id: &str) -> Result<Data, DomainError> {
        let plugin = self.get_plugin().await?;
        plugin.get_data(ctx, id).await.map_err(Into::into)
    }
}
```

### Step 3: Implement a Plugin

Each plugin module:

1. Generates a stable GTS instance ID
2. Registers the plugin **instance** in types-registry (schema is registered by gateway)
3. Registers a scoped client in ClientHub

```rust
// plugins/<vendor>_plugin/src/module.rs

use std::sync::Arc;
use async_trait::async_trait;
use modkit::client_hub::ClientScope;
use modkit::gts::BaseModkitPluginV1;
use modkit::{Module, ModuleCtx};
use modkit_security::SecurityCtx;
use my_sdk::{MyModulePluginClient, MyModulePluginSpecV1};
use types_registry_sdk::TypesRegistryApi;

#[modkit::module(
    name = "vendor_a_plugin",
    deps = ["types_registry"],
)]
pub struct VendorAPlugin {
    service: arc_swap::ArcSwapOption<Service>,
}

#[async_trait]
impl Module for VendorAPlugin {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        let cfg: PluginConfig = ctx.config()?;

        // 1. Generate stable GTS instance ID
        let instance_id = MyModulePluginSpecV1::gts_make_instance_id("vendor_a.pkg_b.my_module.plugin.v1");

        // 2. Register plugin INSTANCE in types-registry
        //    Note: The plugin SCHEMA is registered by the gateway module
        let registry = ctx.client_hub().get::<dyn TypesRegistryApi>()?;
        let instance = BaseModkitPluginV1::<MyModulePluginSpecV1> {
            id: instance_id.clone(),
            vendor: cfg.vendor,
            priority: cfg.priority,
            properties: MyModulePluginSpecV1,
        };
        let instance_json = serde_json::to_value(&instance)?;
        let _ = registry
            .register(&SecurityCtx::root_ctx(), vec![instance_json])
            .await?;

        // 3. Create service and register SCOPED client
        let service = Arc::new(Service::new());
        self.service.store(Some(service.clone()));

        let api: Arc<dyn MyModulePluginClient> = service;
        ctx.client_hub()
            .register_scoped::<dyn MyModulePluginClient>(ClientScope::gts_id(&instance_id), api);

        tracing::info!(instance_id = %instance_id, "Plugin initialized");
        Ok(())
    }
}
```

The plugin service implements the plugin API:

```rust
// plugins/<vendor>_plugin/src/domain/service.rs

use async_trait::async_trait;
use modkit_security::SecurityCtx;
use my_sdk::{Data, MyError, MyModulePluginClient, Query, Page};

pub struct Service;

#[async_trait]
impl MyModulePluginClient for Service {
    async fn get_data(&self, _ctx: &SecurityCtx, id: &str) -> Result<Data, MyError> {
        // Vendor-specific implementation
        Ok(Data { id: id.to_owned(), /* ... */ })
    }

    async fn list_data(&self, _ctx: &SecurityCtx, query: Query) -> Result<Page<Data>, MyError> {
        // Vendor-specific implementation
        todo!()
    }
}
```

---

## Plugin Selection Strategies

The gateway can select plugins based on various criteria:

### By Vendor (Configuration-Based)

```yaml
# config/quickstart.yaml
modules:
  my_gateway:
    config:
      vendor: "Contoso"  # Select Contoso plugin
```

```rust
fn choose_plugin(vendor: &str, instances: &[GtsEntity]) -> Result<&GtsEntity, DomainError> {
    instances
        .iter()
        .filter(|e| {
            let spec: MyModulePluginSpecV1 = serde_json::from_value(e.content.clone()).ok()?;
            spec.vendor == vendor
        })
        .min_by_key(|e| {
            let spec: MyModulePluginSpecV1 = serde_json::from_value(e.content.clone()).unwrap();
            spec.priority
        })
        .ok_or(DomainError::PluginNotFound { vendor: vendor.to_owned() })
}
```

### By Tenant (Context-Based)

```rust
async fn get_plugin_for_tenant(
    &self,
    ctx: &SecurityCtx,
) -> Result<Arc<dyn MyModulePluginClient>, DomainError> {
    // Look up tenant-specific plugin configuration
    let tenant_id = ctx.tenant_id();
    let plugin_id = self.tenant_plugin_map.get(&tenant_id)?;
    let scope = ClientScope::gts_id(plugin_id);
    self.hub.get_scoped::<dyn PluginClient>(&scope)
}
```

### By Request Parameters

```rust
pub async fn handle_request(
    &self,
    ctx: &SecurityCtx,
    provider: &str,  // e.g., "openai", "anthropic"
) -> Result<Response, DomainError> {
    let plugin_id = format!("gts.x.core.modkit.plugin.v1~x.llm_gateway.llm_gateway.plugin.v1~{}.llm_gateway._.plugin.v1", provider);
    let scope = ClientScope::gts_id(&plugin_id);
    let plugin = self.hub.get_scoped::<dyn LlmPluginClient>(&scope)?;
    plugin.complete(ctx, request).await
}
```

---

## Configuration

### Gateway Configuration

```yaml
# config/quickstart.yaml
modules:
  my_gateway:
    config:
      vendor: "Contoso"
      fallback_vendor: "Default"
```

```rust
// <gateway>-gw/src/config.rs
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GatewayConfig {
    pub vendor: String,
    pub fallback_vendor: Option<String>,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            vendor: "Default".to_owned(),
            fallback_vendor: None,
        }
    }
}
```

### Plugin Configuration

```yaml
# config/quickstart.yaml
modules:
  contoso_plugin:
    config:
      vendor: "Contoso"
      priority: 10
  fabrikam_plugin:
    config:
      vendor: "Fabrikam"
      priority: 20  # Lower priority = selected if vendor matches
```

```rust
// plugins/contoso_plugin/src/config.rs
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PluginConfig {
    pub vendor: String,
    pub priority: i16,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            vendor: "Contoso".to_owned(),
            priority: 10,
        }
    }
}
```

---

## Error Handling

### Domain Errors (Gateway)

```rust
// <gateway>-gw/src/domain/error.rs
#[derive(thiserror::Error, Debug)]
pub enum DomainError {
    #[error("types registry unavailable: {0}")]
    TypesRegistryUnavailable(String),

    #[error("no plugin found for vendor '{vendor}'")]
    PluginNotFound { vendor: String },

    #[error("invalid plugin instance '{gts_id}': {reason}")]
    InvalidPluginInstance { gts_id: String, reason: String },

    #[error("plugin client not registered for '{gts_id}'")]
    PluginClientNotFound { gts_id: String },

    #[error(transparent)]
    PluginError(#[from] my_sdk::MyError),
}
```

### SDK Errors (Shared)

```rust
// <gateway>-sdk/src/error.rs
#[derive(thiserror::Error, Debug, Clone)]
pub enum MyError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("internal error: {0}")]
    Internal(String),
}
```

---

## Module Dependencies

Ensure proper initialization order by declaring dependencies:

```rust
// Gateway depends on types_registry AND all plugins
#[modkit::module(
    name = "my_gateway",
    deps = ["types_registry", "plugin_a", "plugin_b", "plugin_c"],
    capabilities = [rest]
)]
pub struct MyGateway { /* ... */ }

// Each plugin depends only on types_registry
#[modkit::module(
    name = "plugin_a",
    deps = ["types_registry"],
)]
pub struct PluginA { /* ... */ }
```

This ensures:

1. `types_registry` initializes first
2. All plugins initialize and register their instances
3. Gateway initializes last and can discover all available plugins

---

## Testing Plugins

### Unit Testing a Plugin

```rust
#[tokio::test]
async fn test_plugin_implementation() {
    let service = Service::new();
    let ctx = SecurityCtx::root_ctx();

    let result = service.get_data(&ctx, "test-id").await;
    assert!(result.is_ok());
}
```

### Integration Testing with Mock Registry

```rust
#[tokio::test]
async fn test_gateway_plugin_resolution() {
    let hub = Arc::new(ClientHub::new());

    // Register mock types-registry
    let mock_registry = Arc::new(MockTypesRegistry::new());
    hub.register::<dyn TypesRegistryApi>(mock_registry);

    // Register mock plugin
    let instance_id = "gts.x.core.modkit.plugin.v1~vendor.pkg.my_module.plugin.v1~fabrikam.test._.plugin.v1";
    let mock_plugin: Arc<dyn MyModulePluginClient> = Arc::new(MockPlugin::new());
    hub.register_scoped::<dyn MyModulePluginClient>(ClientScope::gts_id(instance_id), mock_plugin);

    // Test gateway service
    let svc = Service::new(hub, "Test".to_owned());
    let ctx = SecurityCtx::root_ctx();
    let result = svc.get_data(&ctx, "id").await;
    assert!(result.is_ok());
}
```

---

## Best Practices

### 1. Lazy Plugin Resolution

Resolve the plugin on first use, not during `init()`. This avoids race conditions with types-registry readiness:

```rust
pub struct Service {
    resolved: OnceCell<ClientScope>,  // Cached after first resolution
}
```

### 2. Validate Instance IDs Match

Ensure the GTS instance `content.id` matches the registered `gts_id`:

```rust
if content.id != entity.gts_id {
    return Err(DomainError::InvalidPluginInstance {
        gts_id: entity.gts_id.clone(),
        reason: format!("content.id mismatch: expected {:?}, got {:?}", entity.gts_id, content.id),
    });
}
```

### 3. Use Priority for Fallback

When multiple plugins match, select by priority (lower = higher priority):

```rust
instances.iter()
    .filter(|e| matches_criteria(e))
    .min_by_key(|e| parse_priority(e))
```

### 4. Log Plugin Selection

Always log which plugin was selected for debugging:

```rust
tracing::info!(
    plugin_gts_id = %selected_id,
    vendor = %self.vendor,
    "Selected plugin instance"
);
```

### 5. Handle Plugin Not Found Gracefully

Provide clear error messages when no plugin matches:

```rust
Err(DomainError::PluginNotFound {
    vendor: self.vendor.clone(),
})
```

### 6. Gateway Registers Schema, Plugins Register Instances

Keep schema registration in the gateway module for clear ownership:

| Component | Registers |
|-----------|-----------|
| Gateway | Plugin **schema** (GTS type definition) |
| Each Plugin | Its **instance** (metadata + scoped client) |

This ensures:
- Schema is registered once by the authoritative owner
- Plugins are simpler — they only declare their own existence
- No race conditions or duplicate registration attempts

---

## Example: Tenant Resolver

See `examples/plugin-modules/tenant_resolver/` for a complete working example:

- **`tenant_resolver-sdk/`** — SDK with `TenantResolverClient` (public) and `TenantResolverPluginClient` (plugin) traits
- **`tenant_resolver-gw/`** — Gateway module that registers schema and selects plugin by vendor config
- **`plugins/contoso_tr_plugin/`** — Contoso vendor implementation (registers instance only)
- **`plugins/fabrikam_tr_plugin/`** — Fabrikam vendor implementation (registers instance only)

### Running the Example

```bash
# Build with feature flag
cargo check -p hyperspot-server --features tenant-resolver-example

# Run
cargo run -p hyperspot-server --features tenant-resolver-example -- \
    --config config/quickstart.yaml run

# Test endpoints
curl http://127.0.0.1:8087/tenant-resolver/v1/root
curl http://127.0.0.1:8087/tenant-resolver/v1/tenants?limit=50
```

---

## Further Reading

- [MODKIT_UNIFIED_SYSTEM.md](./MODKIT_UNIFIED_SYSTEM.md) — Complete ModKit architecture guide
- [NEW_MODULE.md](../guidelines/NEW_MODULE.md) — Step-by-step module creation guide
- [ARCHITECTURE_MANIFEST.md](./ARCHITECTURE_MANIFEST.md) — HyperSpot architecture overview
- [types-registry](../modules/system/types-registry/) — GTS types and instances registry
