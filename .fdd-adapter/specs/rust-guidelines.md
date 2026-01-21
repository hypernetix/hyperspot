# Rust Development Guidelines

**Source**: guidelines/DNA/languages/RUST.md, CONTRIBUTING.md

## Backend Stack

**Core Libraries**:
- `axum` - Routing/middleware
- `tower-http` - CORS, compression, timeouts
- `serde` - JSON & validation
- `validator` - Input validation
- `utoipa` - OpenAPI generation
- `utoipa-swagger-ui` - Docs UI (optional)
- `tracing` / `tracing-subscriber` - Observability
- `sqlx` or `sea-orm` - Database access
- `uuid` (v7) or `ulid` - Identifiers
- `time` - Timestamps (prefer over `chrono`)

## Data Types

**Structs**:
```rust
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct Ticket {
    pub id: Uuid,
    pub title: String,
    pub priority: TicketPriority,
    pub status: TicketStatus,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<OffsetDateTime>,
}
```

**Enums**:
```rust
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TicketPriority { 
    Low, 
    Medium, 
    High 
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TicketStatus { 
    Open, 
    InProgress, 
    Resolved, 
    Closed 
}
```

## Timestamps

**Use `time::OffsetDateTime`** (not `chrono`):
- Required fields: `#[serde(with = "time::serde::rfc3339")]`
- Optional fields: `#[serde(with = "time::serde::rfc3339::option")]`
- Always RFC3339 / ISO-8601 formatting with milliseconds

## Durations

**Use `std::time::Duration`** with human-readable parsing:
```rust
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(with = "modkit_utils::humantime_serde::option", default)]
    pub timeout: Duration
}
```

Allows config like: `timeout = "30s"` or `retry_interval = "5m"`

## Error Handling (Problem Details RFC 9457)

```rust
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct Problem {
    #[schema(example = "https://api.example.com/errors/validation")]
    pub r#type: String,
    #[schema(example = "Invalid request")]
    pub title: String,
    pub status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
    pub trace_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<ValidationError>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ValidationError {
    pub field: String,
    pub code: String,
    pub message: String,
}
```

## Axum Handlers with OpenAPI

```rust
use axum::{extract::Query, response::Json};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListParams {
    #[serde(flatten)]
    pub filters: HashMap<String, String>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct ListResponse<T> {
    pub items: Vec<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_info: Option<PageInfo>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct PageInfo {
    pub limit: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_cursor: Option<String>,
}

/// List tickets with cursor pagination
#[utoipa::path(
    get,
    path = "/v1/tickets",
    params(ListParams),
    responses(
        (status = 200, description = "List tickets", body = ListResponse<Ticket>),
        (status = 422, description = "Validation error", body = Problem)
    ),
    security(("oauth2" = []))
)]
pub async fn list_tickets(
    Query(params): Query<ListParams>
) -> impl IntoResponse {
    // Extract OData params
    let limit = params.filters.get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(25);
    let cursor = params.filters.get("cursor").cloned();
    let filter = params.filters.get("$filter").cloned();
    let orderby = params.filters.get("$orderby").cloned();
    let select = params.filters.get("$select").cloned();
    
    // ... database logic ...
}
```

## Input Validation

```rust
use validator::{Validate, ValidationError};

#[derive(Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    
    #[validate(email)]
    pub email: String,
    
    #[validate(custom = "validate_password")]
    pub password: String,
}

fn validate_password(password: &str) -> Result<(), ValidationError> {
    if password.len() < 8 {
        return Err(ValidationError::new("Password too short"));
    }
    Ok(())
}
```

## OpenAPI Generation

**Requirements**:
- Annotate handlers with `utoipa::path`
- Export OpenAPI 3.1 JSON at `/api-docs/openapi.json`
- Validate in CI with OpenAPI linter

## Idempotency & ETags

**Idempotency**:
- Persist `(idempotency_key, request_fingerprint, response_hash, expires_at)`
- On replay with same fingerprint: return stored response + `Idempotency-Replayed: true`

**ETags**:
- For writes, compute and return `ETag`
- Clients send `If-Match` for concurrency control

## Code Style

**Formatting**:
- Use `rustfmt` (standard Rust formatting)
- Command: `cargo fmt --all`
- CI check: `cargo fmt --all -- --check`

**Linting**:
- Use `clippy` (standard Rust linter)
- Command: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- Deny warnings in CI

**Naming**:
- Types: `PascalCase` (struct, enum, trait)
- Functions/Variables: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- Modules: `snake_case`

## Documentation

**Doc Comments**:
- Use `///` for public items
- Use `//!` for module docs
- Include examples when helpful

```rust
/// Gets information about the parser.
///
/// # Example
/// ```
/// let info = get_parser_info().await?;
/// ```
pub async fn get_parser_info() -> Result<Info> { ... }
```

## Error Handling Patterns

**Use Result<T, E> throughout**:
- Never panic in library code
- Domain errors: Custom error types per module
- API errors: Convert to RFC 9457 Problem Details

## Import Organization

**Group imports**:
1. Standard library (`std::...`)
2. External crates
3. Internal crates (`modkit::...`)
4. Local modules (`crate::...`, `super::...`)

```rust
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use modkit::Context;

use crate::contract::FileInfo;
```

## Best Practices

- ✅ Use strong typing throughout
- ✅ Leverage Rust's ownership system
- ✅ Prefer `async/await` for I/O operations
- ✅ Use `tracing` for structured logging
- ✅ Implement `Debug` for all types
- ✅ Use `#[non_exhaustive]` for public enums
- ✅ Avoid `unwrap()` in production code
- ✅ Use `anyhow::Result` for application errors
- ✅ Use `thiserror` for library errors
- ❌ No `unsafe` code without explicit review
- ❌ No panics in library code
- ❌ No `.expect()` without clear invariant explanation

---

## Validation Checklist

### MUST Requirements (Mandatory)

- [ ] **Backend stack used** (axum, serde, utoipa, tracing, sqlx/sea-orm)
- [ ] **Data types follow patterns** (struct/enum with derives)
- [ ] **Timestamps use time::OffsetDateTime** (not chrono)
- [ ] **Durations use std::time::Duration** with humantime serde
- [ ] **Error handling uses Problem Details** (RFC 9457)
- [ ] **Axum handlers annotated** with utoipa::path
- [ ] **Input validation uses validator** crate
- [ ] **OpenAPI generation configured** (utoipa)
- [ ] **Code formatted with rustfmt**
- [ ] **Linting with clippy** (warnings denied)
- [ ] **Naming conventions followed** (PascalCase types, snake_case functions)
- [ ] **Doc comments for public items** (`///`)
- [ ] **Result<T, E> used throughout** (no panics)
- [ ] **Imports organized** (std → external → internal → local)

### SHOULD Requirements (Strongly Recommended)

- [ ] Idempotency support implemented
- [ ] ETags for concurrency control
- [ ] Examples in doc comments
- [ ] Debug derive for all types
- [ ] thiserror for library errors
- [ ] anyhow::Result for application errors

### MAY Requirements (Optional)

- [ ] Custom derive macros
- [ ] Performance optimizations documented
- [ ] Unsafe code with safety comments

## Compliance Criteria

**Pass**: All MUST requirements met (14/14) + cargo check passes  
**Fail**: Any MUST requirement missing or compilation errors

### Agent Instructions

When writing Rust code:
1. ✅ **ALWAYS use specified backend stack** (axum, serde, utoipa, etc.)
2. ✅ **ALWAYS derive required traits** (Serialize, Deserialize, ToSchema, Debug)
3. ✅ **ALWAYS use time::OffsetDateTime** (not chrono)
4. ✅ **ALWAYS use RFC3339 serialization** for timestamps
5. ✅ **ALWAYS include milliseconds** in timestamps
6. ✅ **ALWAYS use Problem Details** for errors
7. ✅ **ALWAYS annotate handlers** with utoipa::path
8. ✅ **ALWAYS validate input** with validator crate
9. ✅ **ALWAYS format with rustfmt** before commit
10. ✅ **ALWAYS resolve clippy warnings** (zero warnings)
11. ✅ **ALWAYS follow naming conventions** (PascalCase/snake_case)
12. ✅ **ALWAYS document public APIs** with `///`
13. ✅ **ALWAYS use Result<T, E>** (no unwrap/panic)
14. ✅ **ALWAYS organize imports** correctly
15. ❌ **NEVER use unwrap()** in production code
16. ❌ **NEVER panic** in library code
17. ❌ **NEVER use chrono** (use time crate)
18. ❌ **NEVER skip validation** (always use validator)

### Rust Code Review Checklist

Before committing:
- [ ] All derives present (Serialize, Deserialize, ToSchema, Debug)
- [ ] Timestamps use time::OffsetDateTime with RFC3339
- [ ] Errors use Problem Details struct
- [ ] Handlers have utoipa::path annotations
- [ ] Input structs have validator derives
- [ ] OpenAPI generation configured
- [ ] rustfmt applied (`cargo fmt --all`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Naming conventions followed
- [ ] Public items documented
- [ ] No unwrap/panic in library code
- [ ] Imports organized
- [ ] cargo check passes
- [ ] Tests written and passing
