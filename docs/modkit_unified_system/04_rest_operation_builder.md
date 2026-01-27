# REST API with OperationBuilder

ModKit provides a type-safe operation builder that prevents half-wired routes at compile time and integrates with OpenAPI, auth, errors, SSE, and content types.

## Core principles

- **Rule**: Strictly follow the API guideline (`guidelines/DNA/REST/API.md`).
- **Rule**: Do NOT implement a REST host. `api_gateway` owns the Axum server and OpenAPI. Modules only register routes via `register_routes(...)`.
- **Rule**: Use `Extension<Arc<Service>>` for dependency injection and attach the service ONCE after all routes are registered: `router = router.layer(Extension(service.clone()));`.
- **Rule**: Use `Authz(ctx): Authz` extractor for authorization — it extracts `SecurityContext` from the request.
- **Rule**: Follow the `<crate>.<resource>.<action>` convention for `operation_id` naming.
- **Rule**: Use `modkit::api::prelude::*` for ergonomic handler types (ApiResult, created_json, no_content).
- **Rule**: Always return RFC 9457 Problem Details for all 4xx/5xx errors via `Problem` (implements `IntoResponse`).
- **Rule**: Observability is provided by gateway: request tracing and `X-Request-Id` are already handled.
- **Rule**: Do not add transport middlewares (CORS, timeouts, compression, body limits) at module level.
- **Rule**: Handlers should complete within ~30s (gateway timeout). If work may exceed that, return `202 Accepted`.

## OperationBuilder basics

```rust
use modkit::api::{OperationBuilder, OpenApiRegistry};
use modkit::api::operation_builder::{LicenseFeature, OperationBuilderODataExt};
use axum::{Extension, Router};
use std::sync::Arc;

pub fn register_routes(
    router: Router,
    openapi: &dyn OpenApiRegistry,
    service: Arc<Service>,
) -> Router {
    let router = OperationBuilder::get("/users-info/v1/users")
        .operation_id("users_info.list_users")
        .require_auth(&Resource::Users, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::list_users)
        .json_response_with_schema::<modkit_odata::Page<dto::UserDto>>(
            openapi,
            http::StatusCode::OK,
            "Paginated list of users",
        )
        .with_odata_filter::<dto::UserDtoFilterField>()
        .with_odata_select()
        .with_odata_orderby::<dto::UserDtoFilterField>()
        .standard_errors(openapi)
        .register(router, openapi);

    // Attach service once after all routes
    router.layer(Extension(service))
}
```

## Common patterns

### Auth-protected endpoint

```rust
OperationBuilder::post("/users-info/v1/users")
    .operation_id("users_info.create_user")
    .require_auth(&Resource::Users, &Action::Write)
    .handler(handlers::create_user)
    .json_response_with_schema::<dto::UserDto>(openapi, StatusCode::CREATED, "User created")
    .standard_errors(openapi)
    .register(router, openapi);
```

### Public endpoint

```rust
OperationBuilder::get("/users-info/v1/health")
    .operation_id("users_info.health")
    .public()
    .handler(handlers::health)
    .json_response_with_schema::<dto::HealthDto>(openapi, StatusCode::OK, "Health check")
    .register(router, openapi);
```

### License-gated endpoint

```rust
OperationBuilder::get("/users-info/v1/premium")
    .operation_id("users_info.premium")
    .require_auth(&Resource::Users, &Action::Read)
    .require_license_features::<License>([License::Premium])
    .handler(handlers::premium)
    .json_response_with_schema::<dto::PremiumDto>(openapi, StatusCode::OK, "Premium content")
    .standard_errors(openapi)
    .register(router, openapi);
```

## Content types

### JSON request/response

```rust
OperationBuilder::post("/users-info/v1/users")
    .json_request::<CreateUserReq>(openapi, "User creation data")
    .handler(handlers::create_user)
    .json_response_with_schema::<UserDto>(openapi, StatusCode::CREATED, "User created")
    .standard_errors(openapi)
    .register(router, openapi);
```

### Multipart file upload

```rust
OperationBuilder::post("/users-info/v1/upload")
    .multipart_file_request("file", Some("File to upload"))
    .handler(handlers::upload)
    .json_response_with_schema::<UploadResponse>(openapi, StatusCode::OK, "Upload successful")
    .standard_errors(openapi)
    .register(router, openapi);
```

### Binary/octet-stream

```rust
OperationBuilder::post("/users-info/v1/parse")
    .octet_stream_request(Some("Raw file bytes"))
    .handler(handlers::parse_bytes)
    .json_response_with_schema::<ParseResponse>(openapi, StatusCode::OK, "Parse successful")
    .standard_errors(openapi)
    .register(router, openapi);
```

### Custom content types

```rust
OperationBuilder::post("/users-info/v1/export")
    .allow_content_types(&["application/pdf", "image/png", "image/jpeg"])
    .handler(handlers::export)
    .binary_response(openapi, StatusCode::OK, "Exported file")
    .standard_errors(openapi)
    .register(router, openapi);
```

## Server-Sent Events (SSE)

```rust
OperationBuilder::get("/users-info/v1/users/events")
    .operation_id("users_info.user_events")
    .require_auth(&Resource::Users, &Action::Read)
    .sse_json::<dto::UserEvent>(openapi, "Real-time user events")
    .handler(handlers::user_events)
    .register(router, openapi);
```

Handler example:

```rust
pub async fn user_events(
    Authz(ctx): Authz,
    Extension(broadcaster): Extension<Arc<SseBroadcaster<UserEvent>>>,
) -> impl IntoResponse {
    let stream = broadcaster.subscribe();
    Sse::new(stream, "user_events")
}
```

## Error handling

### Standard errors

```rust
.standard_errors(openapi)
```

Adds: 400, 401, 403, 404, 409, 422, 429, 500.

### Specific errors

```rust
.problem_response(openapi, StatusCode::BAD_REQUEST, "Invalid input")
.problem_response(openapi, StatusCode::CONFLICT, "Email already exists")
.problem_response(openapi, StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
.with_422_validation_error(openapi)
```

### Handler error pattern

```rust
use modkit::api::prelude::*;
use modkit::api::problem::Problem;
use crate::domain::error::DomainError;

// DomainError auto-converts to Problem via From impl
pub async fn create_user(
    Authz(ctx): Authz,
    Extension(svc): Extension<Arc<Service>>,
    Json(req): Json<CreateUserReq>,
) -> ApiResult<impl IntoResponse> {
    let user = svc.create_user(&ctx, req.into()).await?;
    let id_str = user.id.to_string();
    Ok(created_json(UserDto::from(user), &uri, &id_str))
}
```

## OData integration

### OData-enabled list endpoint

```rust
OperationBuilder::get("/users-info/v1/users")
    .operation_id("users_info.list_users")
    .require_auth(&Resource::Users, &Action::Read)
    .handler(handlers::list_users)
    .json_response_with_schema::<modkit_odata::Page<dto::UserDto>>(
        openapi,
        StatusCode::OK,
        "Paginated list of users",
    )
    .with_odata_filter::<dto::UserDtoFilterField>()
    .with_odata_select()
    .with_odata_orderby::<dto::UserDtoFilterField>()
    .standard_errors(openapi)
    .register(router, openapi);
```

Handler with OData:

```rust
pub async fn list_users(
    Authz(ctx): Authz,
    Extension(svc): Extension<Arc<Service>>,
    OData(query): OData,
) -> ApiResult<JsonPage<serde_json::Value>> {
    let page: modkit_odata::Page<user_info_sdk::User> =
        svc.users.list_users_page(&ctx, &query).await?;
    let page = page.map_items(UserDto::from);
    Ok(Json(page_to_projected_json(&page, query.selected_fields())))
}
```

## OpenAPI registration

### Schema-aware responses

```rust
.json_response_with_schema::<T>(openapi, StatusCode::OK, "Success")
```

### Error schemas

```rust
.error_400(openapi)
.error_401(openapi)
.error_403(openapi)
.error_404(openapi)
.error_409(openapi)
.error_422(openapi)
.error_500(openapi)
```

### SSE schema

```rust
.sse_json::<T>(openapi, "Real-time event stream")
```

## Handler return types

| Pattern | Return Type | Helper |
|---------|-------------|--------|
| GET with body | `ApiResult<JsonBody<T>>` | `Ok(Json(dto))` |
| POST with body | `ApiResult<impl IntoResponse>` | `Ok(created_json(dto, location))` |
| DELETE no body | `ApiResult<impl IntoResponse>` | `Ok(no_content())` |
| Paginated list | `ApiResult<JsonPage<T>>` | `Ok(Json(page))` |
| Binary | `ApiResult<impl IntoResponse>` | `Ok(Response::new(...))` |

## Quick checklist

- [ ] Use `OperationBuilder` for every route.
- [ ] Add `.require_auth()` for protected endpoints.
- [ ] Add `.standard_errors(openapi)` or specific errors.
- [ ] Use `.json_response_with_schema()` for typed responses.
- [ ] Use `Extension<Arc<Service>>` and attach once after all routes.
- [ ] Use `Authz(ctx): Authz` to get `SecurityContext`.
- [ ] Use `ApiResult<T>` and `?` for error propagation.
- [ ] For OData: add `.with_odata_*()` helpers and use `OData(query)` extractor.
- [ ] For SSE: use `.sse_json()` and `SseBroadcaster`.
