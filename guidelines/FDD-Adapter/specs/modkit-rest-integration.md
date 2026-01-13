# ModKit REST Integration - MANDATORY Specification

**Version**: 1.0  
**Status**: REQUIRED for all REST modules  
**Last Updated**: 2026-01-08

---

## Overview

**Critical**: All REST modules MUST integrate through ModKit's `RestfulModule` trait and `OperationBuilder` pattern. Direct axum integration is **FORBIDDEN**.

**Why**: 
- `api_gateway` provides automatic middleware (JWT, SecurityCtx, tracing)
- `OperationBuilder` ensures type-safety and OpenAPI generation
- Direct axum bypasses platform features and breaks observability

---

## MANDATORY Pattern

### 1. Module Declaration

```rust
#[modkit::module(
    name = "my_module",
    capabilities = [rest],  // ← Adds api_gateway dependency automatically
    deps = []
)]
pub struct MyModule {
    service: Arc<MyService>,
}
```

**Note**: `api_gateway` dependency added automatically when `rest` capability declared.

---

### 2. RestfulModule Implementation

```rust
use modkit::{Module, RestfulModule, ModuleCtx};
use modkit::api::OpenApiRegistry;
use axum::Router;

impl RestfulModule for MyModule {
    fn register_rest(
        &self,
        _ctx: &ModuleCtx,
        router: Router,                      // ← Extend this router
        openapi: &dyn OpenApiRegistry,       // ← Register operations here
    ) -> anyhow::Result<Router> {
        // Call routes module to register endpoints
        let router = crate::api::rest::routes::register_routes(
            router,
            openapi,
            self.service.clone()
        );
        Ok(router)
    }
}
```

**MUST**:
- ✅ Extend passed `router` parameter
- ✅ Register operations through `openapi` parameter
- ✅ Return extended router

**FORBIDDEN**:
- ❌ `Router::new()` - creates isolated router
- ❌ Ignoring `router` parameter
- ❌ Not using `openapi` parameter

---

### 3. Routes Registration (OperationBuilder)

```rust
// api/rest/routes.rs
use axum::Router;
use modkit::api::{OpenApiRegistry, OperationBuilder};

pub fn register_routes(
    mut router: Router,
    openapi: &dyn OpenApiRegistry,
    service: Arc<MyService>
) -> Router {
    // Register each endpoint with OperationBuilder
    router = OperationBuilder::get("/my-module/v1/resource/{id}")
        .operation_id("my_module.get_resource")
        .summary("Get resource by ID")
        .tag("My Module")
        .path_param("id", "Resource ID")
        .require_auth(&Resource::MyResource, &Action::Read)
        .handler(handlers::get_resource)
        .json_response_with_schema::<ResourceDto>(
            openapi,
            http::StatusCode::OK,
            "Resource retrieved"
        )
        .standard_errors(openapi)
        .register(router, openapi);  // ← Returns extended router
    
    // Attach service state via Extension layer
    router = router.layer(Extension(service));
    
    router
}
```

**MUST**:
- ✅ Use `OperationBuilder::get/post/put/patch/delete`
- ✅ Set `.operation_id()` (unique identifier)
- ✅ Set `.require_auth()` or `.public()`
- ✅ Set `.handler()` with actual handler function
- ✅ Add at least one `.json_response_with_schema()` or response method
- ✅ Call `.register(router, openapi)` to finalize
- ✅ Pass `openapi` registry for schema registration

**FORBIDDEN**:
- ❌ Direct `axum::routing::get()` without OperationBuilder
- ❌ `Router::new()` creating new router
- ❌ Manual route registration with `.route()`
- ❌ Missing OpenAPI documentation

---

### 4. Handler Implementation

```rust
// api/rest/handlers.rs
use axum::{Extension, extract::Path, Json};
use modkit_security::SecurityCtx;

pub async fn get_resource(
    Path(id): Path<String>,
    Extension(ctx): Extension<SecurityCtx>,     // ← Injected by api_gateway
    Extension(service): Extension<Arc<MyService>>
) -> Result<Json<ResourceDto>, Problem> {
    // SecurityCtx already validated and injected
    let resource = service.get_resource(&id, &ctx).await?;
    Ok(Json(resource.into()))
}
```

**MUST**:
- ✅ Extract `SecurityCtx` via `Extension` (auto-injected)
- ✅ Use `Problem` for error responses (RFC 7807)
- ✅ Return typed responses matching OperationBuilder schema

**AUTOMATIC** (provided by api_gateway):
- ✅ JWT validation (happens before handler)
- ✅ SecurityCtx injection (available in Extension)
- ✅ Request tracing (correlation IDs added)
- ✅ Error handling (Problem Details serialization)

---

## What api_gateway Provides Automatically

When you implement `RestfulModule`, `api_gateway` automatically applies:

### 1. Authentication Middleware
- JWT signature validation
- Claims extraction
- Token expiration check
- **Access**: `Extension<Claims>` in handler

### 2. Security Context Injection
- Tenant ID extraction from JWT
- Scope creation (tenant/global)
- Subject identification
- **Access**: `Extension<SecurityCtx>` in handler

### 3. Request Tracing
- Correlation ID generation
- Distributed tracing headers
- Request/response logging
- **Access**: Automatic via tracing crate

### 4. Error Handling
- Problem Details (RFC 7807) serialization
- Consistent error format
- Status code mapping
- **Access**: Return `Problem` from handler

### 5. OpenAPI Generation
- `/openapi.json` endpoint
- Swagger UI at `/swagger-ui`
- Schema registry
- **Access**: Via `OperationBuilder` registration

---

## Anti-Patterns (FORBIDDEN)

### ❌ Anti-Pattern 1: Manual Middleware

```rust
// ❌ WRONG - Reimplementing platform middleware
pub async fn jwt_validation_middleware(req: Request, next: Next) -> Response {
    // Validation logic...
}

Router::new()
    .layer(from_fn(jwt_validation_middleware))  // ❌ Duplicate platform
```

**Why wrong**: `api_gateway` already validates JWT for all REST modules.

**Fix**: Remove middleware, use `Extension<SecurityCtx>` in handler.

---

### ❌ Anti-Pattern 2: Direct Axum Routes

```rust
// ❌ WRONG - Bypassing OperationBuilder
pub fn create_router() -> Router {
    Router::new()  // ❌ New router instead of extending
        .route("/gts/{id}", axum::routing::get(handler))  // ❌ No OpenAPI
        .route("/gts/{id}", axum::routing::post(handler))
}
```

**Why wrong**: 
- No OpenAPI documentation
- No type-safety
- Bypasses api_gateway middleware
- Not discoverable

**Fix**: Use `OperationBuilder` in `RestfulModule::register_rest`.

---

### ❌ Anti-Pattern 3: Custom OData Parsing

```rust
// ❌ WRONG - Reimplementing OData parsing
fn parse_odata_params(query: &str) -> ODataParams {
    // Manual parsing...
}
```

**Why wrong**: `modkit-odata` provides validated OData parsing.

**Fix**: Use `modkit_odata::ODataQuery` and platform builders.

---

## Validation Checklist

When implementing REST integration, verify:

### Module Level
- [ ] `#[modkit::module(capabilities = [rest])]` declared
- [ ] `impl RestfulModule for MyModule` exists
- [ ] `register_rest()` extends passed router, not creates new
- [ ] `register_rest()` returns extended router

### Routes Level  
- [ ] All endpoints use `OperationBuilder`
- [ ] Each operation has `.operation_id()`
- [ ] Each operation has `.handler()`
- [ ] Each operation has response definition
- [ ] Each operation calls `.register(router, openapi)`
- [ ] No direct `axum::routing::*` calls
- [ ] No `Router::new()` in routes.rs

### Handler Level
- [ ] Uses `Extension<SecurityCtx>` for auth context
- [ ] Returns `Result<T, Problem>` for errors
- [ ] No manual JWT validation
- [ ] No manual SecurityCtx creation

### Testing
- [ ] Integration tests use `api_gateway` test helpers
- [ ] Tests verify OpenAPI registration
- [ ] Tests verify middleware chain

---

## Example: Complete Implementation

See working examples:
- `@/modules/file_parser/src/module.rs` - RestfulModule impl
- `@/modules/file_parser/src/api/rest/routes.rs` - OperationBuilder usage
- `@/modules/nodes_registry/src/api/rest/routes.rs` - Auth patterns

---

## References

- ModKit docs: `@/docs/MODKIT_UNIFIED_SYSTEM.md`
- OperationBuilder example: `@/examples/modkit/type_safe_api_builder.rs`
- Platform conventions: `@/guidelines/FDD-Adapter/specs/conventions.md`
- REST API guidelines: `@/guidelines/DNA/REST/API.md`

---

## Migration from Anti-Patterns

If you have existing code with anti-patterns:

1. **Remove custom middleware** (`middleware.rs` with JWT/SecurityCtx)
2. **Replace direct axum** with OperationBuilder in `routes.rs`
3. **Implement RestfulModule** in `module.rs`
4. **Update handlers** to use `Extension<SecurityCtx>`
5. **Remove** custom Router creation
6. **Test** that OpenAPI docs appear at `/openapi.json`
