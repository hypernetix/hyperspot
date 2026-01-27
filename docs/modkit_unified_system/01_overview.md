# ModKit Overview

## What ModKit provides

- **Composable modules** discovered via `inventory`, initialized in dependency order.
- **Gateway as a module** (e.g., `api_gateway`) that owns the Axum router and OpenAPI document.
- **Type-safe REST** via an operation builder that prevents half-wired routes at compile time.
- **Server-Sent Events (SSE)** with type-safe broadcasters and domain event integration.
- **OpenAPI 3.1** generation using `utoipa` with automatic schema registration for DTOs.
- **Standardized HTTP errors** with RFC-9457 `Problem` (implements `IntoResponse` directly).
- **Typed ClientHub** for in-process clients (resolve by interface type + optional scope).
- **Plugin architecture** via scoped ClientHub registration and GTS-based discovery (see `docs/MODKIT_PLUGINS.md`).
- **Lifecycle** helpers and wrappers for long-running tasks and graceful shutdown.
- **Lock-free hot paths** via atomic `Arc` swaps for read-mostly state.

## Core invariants (apply everywhere)

- **SDK pattern is the public API**: Use `<module>-sdk` crate for traits, models, errors. Do not expose internals.
- **Secure-by-default DB access**: Use `SecureConn` + `SecurityCtx`. Raw access only for migrations/admin tools and requires `insecure-escape`.
- **RFC-9457 errors everywhere**: Use `Problem` (implements `IntoResponse`). Do not use `ProblemResponse`.
- **Type-safe REST**: Use `OperationBuilder` with `.require_auth()` and `.standard_errors()`.
- **OData macros are in `modkit-odata-macros`**: Use `modkit_odata_macros::ODataFilterable`.
- **ClientHub registration**: `ctx.client_hub().register::<dyn MyModuleApi>(api)`; `ctx.client_hub().get::<dyn MyModuleApi>()?`.
- **Cancellation**: Pass `CancellationToken` to background tasks for cooperative shutdown.
- **GTS schema**: Use `gts_schema_with_refs_as_string()` for faster, correct schema generation.

## Golden path (quick reference)

```rust
// Module registration
#[modkit::module(
    name = "my_module",
    deps = ["foo", "bar"],
    capabilities = [db, rest],
    client = my_module_sdk::MyModuleApi,
)]
pub struct MyModule { /* ... */ }

// Secure DB access
let secure_conn = db.sea_secure();
let users = secure_conn.find::<user::Entity>(&ctx)?.all(secure_conn.conn()).await?;

// ClientHub
ctx.client_hub().register::<dyn my_module_sdk::MyModuleApi>(api);
let api = ctx.client_hub().get::<dyn my_module_sdk::MyModuleApi>()?;

// Errors
impl From<DomainError> for Problem { /* ... */ }
fn handler() -> ApiResult<T> { /* ... */ }

// REST wiring
OperationBuilder::get("/users")
    .require_auth(&Resource::Users, &Action::Read)
    .handler(handler)
    .json_response_with_schema::<UserDto>(openapi, StatusCode::OK, "Users")
    .standard_errors(openapi)
    .register(router, openapi);
```

## Non-goals

- Do not maintain two divergent truths. This folder is the source of truth.
- Do not load the entire folder at once; use selective, task-driven loading.
- Do not mix transport concerns in SDK crates.
- Do not bypass SecureConn without explicit justification and `insecure-escape`.
