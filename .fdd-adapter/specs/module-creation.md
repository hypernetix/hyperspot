# Module Creation Guide

**Source**: guidelines/NEW_MODULE.md, docs/MODKIT_UNIFIED_SYSTEM.md, docs/MODKIT_PLUGINS.md

## ModKit Core Concepts

**ModKit provides**:
- **Composable Modules**: Discovered via `inventory`, initialized in dependency order
- **Gateway as Module**: `api_gateway` owns Axum router and OpenAPI document
- **Type-Safe REST**: Operation builder prevents half-wired routes at compile time
- **Server-Sent Events (SSE)**: Type-safe broadcasters for real-time events
- **Standardized HTTP Errors**: Built-in RFC-9457 `Problem` support
- **Typed ClientHub**: In-process clients resolved by interface type
- **Plugin Architecture**: Scoped ClientHub + GTS-based discovery
- **Lifecycle Management**: Helpers for long-running tasks and graceful shutdown

## SDK Pattern (Required)

Every module MUST follow the SDK pattern:

**Two Crates**:
1. **`<module>-sdk`**: Public API (trait, models, errors) - transport-agnostic
2. **`<module>`**: Implementation (domain logic, REST handlers, local client, infra)

**Benefits**:
- Clear separation between public API and implementation
- Consumers only need lightweight SDK dependency
- Direct ClientHub registration: `hub.get::<dyn MyModuleApi>()?`

## Canonical Directory Structure

```
modules/<your-module>/
├─ <your-module>-sdk/           # SDK crate (public API)
│  ├─ Cargo.toml
│  └─ src/
│     ├─ lib.rs                 # Re-exports
│     ├─ api.rs                 # API trait (all methods take &SecurityCtx)
│     ├─ models.rs              # Transport-agnostic models (NO serde)
│     └─ errors.rs              # Transport-agnostic errors
│
└─ <your-module>/               # Implementation crate
   ├─ Cargo.toml
   └─ src/
      ├─ lib.rs                 # Re-exports SDK + module struct
      ├─ module.rs              # Module struct, #[modkit::module]
      ├─ config.rs              # Typed config with defaults
      ├─ local_client.rs        # Local client implementing SDK API
      ├─ api/                   # Transport adapters
      │  └─ rest/               # HTTP REST layer
      │     ├─ dto.rs           # DTOs (serde, ToSchema)
      │     ├─ handlers.rs      # Thin Axum handlers
      │     ├─ routes.rs        # OperationBuilder registrations
      │     ├─ error.rs         # Problem mapping
      │     └─ sse_adapter.rs   # SSE publisher (optional)
      ├─ domain/                # Business logic
      │  ├─ error.rs            # Domain errors
      │  ├─ events.rs           # Domain events
      │  ├─ ports.rs            # Output ports
      │  ├─ repo.rs             # Repository traits
      │  └─ service.rs          # Service orchestration
      └─ infra/                 # Infrastructure adapters
         └─ storage/            # Database layer
            ├─ entity.rs        # SeaORM entities
            ├─ mapper.rs        # Model<->Entity conversion
            ├─ sea_orm_repo.rs  # Repository implementation
            └─ migrations/      # SeaORM migrations
```

## Layer Responsibilities (DDD-Light)

**API Layer** (`api/rest/`):
- DTOs with serde/utoipa derives
- Thin Axum handlers (web controllers)
- Request/response transformation
- Problem Details mapping
- **Rule**: DTOs only in this layer, not referenced outside

**Domain Layer** (`domain/`):
- Rich domain models
- Business logic
- Domain errors
- Repository traits (ports)
- **Rule**: No external dependencies (DB, HTTP, etc.)

**Infrastructure Layer** (`infra/`):
- Database entities (SeaORM)
- Repository implementations
- External service adapters
- **Rule**: Implements domain ports

## Module Declaration

```rust
#[modkit::module(
    name = "my_module",
    deps = ["dependency_module"],
    capabilities = [db, rest, stateful],
    client = contract::client::MyModuleApi,
    ctor = MyModule::new
)]
pub struct MyModule {
    // State fields
}
```

**Attributes**:
- `name`: Module identifier (required)
- `deps`: Dependency modules (optional, `api_gateway` auto-added for REST)
- `capabilities`: `[db, rest, stateful, rest_host]`
- `client`: API trait for ClientHub registration
- `ctor`: Constructor expression (default: `Default::default()`)

## Capabilities

**`db`**: Module uses database
- Implements `modkit::Db` trait
- Provides migrations

**`rest`**: Module provides REST API
- Implements `modkit::Rest` trait
- Registers routes with `api_gateway`

**`stateful`**: Module has lifecycle
- Implements `modkit::Stateful` trait
- Background tasks, cleanup

**`rest_host`**: Module owns HTTP server
- Only `api_gateway` should use this

## Type-Safe REST with OperationBuilder

```rust
use modkit::api::OperationBuilder;

impl MyModule {
    pub fn register_routes(&self, gateway: &mut dyn modkit::OperationRegistry) {
        OperationBuilder::get("/my-module/v1/items")
            .with_handler(handlers::list_items)
            .with_openapi(|op| {
                op.summary("List items")
                  .tag("MyModule")
            })
            .register(gateway);
    }
}
```

**Benefits**:
- Compile-time route validation
- Automatic OpenAPI generation
- Type-safe handler registration

## ClientHub Pattern

**Publishing** (in SDK crate):
```rust
#[async_trait]
pub trait MyModuleApi: Send + Sync {
    async fn get_item(&self, ctx: &SecurityCtx, id: Uuid) 
        -> Result<Item, MyModuleError>;
}
```

**Local Client** (in implementation):
```rust
pub struct LocalMyModuleClient {
    service: Arc<MyModuleService>,
}

#[async_trait]
impl MyModuleApi for LocalMyModuleClient {
    async fn get_item(&self, ctx: &SecurityCtx, id: Uuid) 
        -> Result<Item, MyModuleError> {
        self.service.get_item(ctx, id).await
    }
}
```

**Registration**:
```rust
ctx.client_hub.register_arc::<dyn MyModuleApi>(
    Arc::new(LocalMyModuleClient::new(service))
)?;
```

**Consumption**:
```rust
let client = ctx.client_hub.get::<dyn MyModuleApi>()?;
let item = client.get_item(&security_ctx, id).await?;
```

## Security Context (Required)

**All API methods MUST accept `&SecurityCtx`**:
```rust
pub async fn get_item(
    &self, 
    ctx: &SecurityCtx,  // ← Required
    id: Uuid
) -> Result<Item, MyModuleError>;
```

This enables:
- Secure ORM integration
- Multi-tenancy enforcement
- Access control

## Configuration

```rust
#[derive(serde::Deserialize, Default, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct MyModuleConfig {
    pub cache_size: usize,
    #[serde(with = "modkit_utils::humantime_serde::option", default)]
    pub timeout: Option<Duration>,
}
```

**Access**:
```rust
let config = ctx.get_typed_config::<MyModuleConfig>()?;
```

## Lifecycle Management

```rust
#[async_trait]
impl Stateful for MyModule {
    async fn start(&mut self, ctx: &ModuleCtx) -> Result<()> {
        // Start background tasks
        let token = ctx.cancellation_token.child_token();
        tokio::spawn(async move {
            // Task that respects cancellation
        });
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<()> {
        // Graceful cleanup
        Ok(())
    }
}
```

## Testing

```rust
#[cfg(test)]
mod tests {
    use tower::ServiceExt;
    
    #[tokio::test]
    async fn test_list_items() {
        let app = test_app().await;
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/my-module/v1/items")
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();
            
        assert_eq!(response.status(), StatusCode::OK);
    }
}
```

## Best Practices

- ✅ Always use SDK pattern (separate API and implementation)
- ✅ All API methods take `&SecurityCtx` first parameter
- ✅ Use `#[modkit::module]` macro for registration
- ✅ Implement all required traits (Module, Db, Rest, Stateful)
- ✅ Use OperationBuilder for type-safe REST
- ✅ Register clients with ClientHub
- ✅ Respect cancellation tokens in background tasks
- ✅ Use Problem Details for HTTP errors
- ✅ Follow DDD-light layering (API/Domain/Infra)
- ❌ Don't reference DTOs outside API layer
- ❌ Don't use serde in SDK models (transport-agnostic)
- ❌ Don't bypass Secure ORM (no raw queries without SecurityCtx)
- ❌ Don't store secrets in code (use config/env vars)

## Reference

See complete examples:
- `examples/modkit/users_info/` - Full SDK pattern implementation
- `modules/system/api_gateway/` - Gateway module (REST host)
- `modules/file_parser/` - Simple module example

---

## Validation Checklist

### MUST Requirements (Mandatory)

- [ ] **SDK pattern followed** (separate -sdk crate)
- [ ] **Canonical directory structure** (lib.rs, module.rs, contract/, api/, domain/, infra/)
- [ ] **Module macro used** (#[modkit::module])
- [ ] **All API methods take &SecurityCtx** first parameter
- [ ] **Capabilities declared** (db, rest, stateful)
- [ ] **ClientHub registration** for SDK API trait
- [ ] **Type-safe REST** with OperationBuilder
- [ ] **DDD-light layering** (API → Domain → Infra)
- [ ] **DTOs isolated to API layer**
- [ ] **Domain layer DB-agnostic**
- [ ] **Configuration typed** with serde derives
- [ ] **Lifecycle management** (graceful shutdown, cancellation tokens)
- [ ] **Testing included** (unit, integration)

### SHOULD Requirements (Strongly Recommended)

- [ ] Local client implements SDK trait
- [ ] SSE adapter for events (if needed)
- [ ] Problem Details mapping for errors
- [ ] Module-specific config section
- [ ] Examples in documentation
- [ ] Migration files for DB schema

### MAY Requirements (Optional)

- [ ] OoP module variant (gRPC)
- [ ] Plugin architecture support
- [ ] Custom validation logic
- [ ] Performance optimizations

## Compliance Criteria

**Pass**: All MUST requirements met (13/13) + module compiles  
**Fail**: Any MUST requirement missing or module doesn't compile

### Agent Instructions

When creating modules:
1. ✅ **ALWAYS use SDK pattern** (separate -sdk and implementation crates)
2. ✅ **ALWAYS follow canonical structure** (contract/, api/, domain/, infra/)
3. ✅ **ALWAYS use #[modkit::module]** macro
4. ✅ **ALWAYS pass &SecurityCtx** to all API methods (first parameter)
5. ✅ **ALWAYS declare capabilities** (db, rest, stateful as needed)
6. ✅ **ALWAYS register with ClientHub**
7. ✅ **ALWAYS use OperationBuilder** for REST routes
8. ✅ **ALWAYS separate layers** (API/Domain/Infra)
9. ✅ **ALWAYS keep DTOs in api/rest/** (not in domain)
10. ✅ **ALWAYS make domain DB-agnostic**
11. ✅ **ALWAYS use typed config** (serde Deserialize)
12. ✅ **ALWAYS respect cancellation** tokens
13. ✅ **ALWAYS write tests**
14. ❌ **NEVER mix layers** (domain accessing DB directly)
15. ❌ **NEVER skip SDK pattern** (always separate API)
16. ❌ **NEVER use serde in SDK contracts** (transport-agnostic)
17. ❌ **NEVER bypass Secure ORM** (always use SecurityCtx)

### Module Creation Checklist

Before implementing module:
- [ ] SDK crate created with API trait
- [ ] Implementation crate created
- [ ] Canonical structure in place (contract/, api/, domain/, infra/)
- [ ] Module macro with correct attributes
- [ ] All API methods have &SecurityCtx
- [ ] Capabilities declared correctly
- [ ] ClientHub registration code
- [ ] OperationBuilder for routes
- [ ] DTOs only in api/rest/
- [ ] Domain models in domain/
- [ ] DB access only in infra/
- [ ] Typed config struct
- [ ] Lifecycle methods implemented
- [ ] Tests written
- [ ] cargo check passes
- [ ] Documentation complete
