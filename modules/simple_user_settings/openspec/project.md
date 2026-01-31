# Project Context

## Purpose

This module is part of **CyberFabric** — a modular, high-performance AI services platform built in Rust.

> Module-specific purpose will be defined in the module spec.

## Tech Stack

- **Language**: Rust (see [`@/guidelines/DNA/languages/RUST.md`](../../../guidelines/DNA/languages/RUST.md))
- **Framework**: ModKit (see [`@/docs/modkit_unified_system/README.md`](../../../docs/modkit_unified_system/README.md))
- **HTTP**: Axum with `tower-http` middleware
- **Database**: SeaORM / SQLx with secure ORM layer (see [`@/docs/modkit_unified_system/06_secure_orm_db_access.md`](../../../docs/modkit_unified_system/06_secure_orm_db_access.md))
- **Observability**: `tracing` + OpenTelemetry (see [`@/docs/TRACING_SETUP.md`](../../../docs/TRACING_SETUP.md))
- **OpenAPI**: `utoipa` for automatic documentation generation
- **IDs**: UUIDv7 for all identifiers
- **Time**: `chrono` with UTC timestamps

## Project Conventions

### Architecture

Follow the architecture manifest: [`@/docs/ARCHITECTURE_MANIFEST.md`](../../../docs/ARCHITECTURE_MANIFEST.md)

**Key principles:**
- **Everything is a Module** — composable, independent units
- **DDD-light structure** — API / Contract / Domain / Infra layers
- **SDK pattern** — separate `-sdk` crate for public API surface
- **Type-safe REST** — OperationBuilder prevents half-wired routes at compile time

### Module Structure

Follow the new module guideline: [`@/guidelines/NEW_MODULE.md`](../../../guidelines/NEW_MODULE.md)

```text
modules/<module>/
├─ <module>-sdk/           # Public API: trait, models, errors (NO serde)
│  └─ src/
│     ├─ api.rs            # API trait (all methods take &SecurityContext)
│     ├─ models.rs         # Transport-agnostic models
│     └─ errors.rs         # Transport-agnostic errors
└─ <module>/               # Implementation
   └─ src/
      ├─ module.rs         # #[modkit::module] declaration
      ├─ local_client.rs   # Local client implementing SDK trait
      ├─ api/rest/         # DTOs, handlers, routes, error mapping
      ├─ domain/           # Business logic, events, repository traits
      └─ infra/storage/    # SeaORM entities, migrations
```

### Code Style

See [`@/guidelines/DNA/languages/RUST.md`](../../../guidelines/DNA/languages/RUST.md)

- **Formatting**: `cargo fmt` with max line length 100, 4-space indentation
- **Linting**: `cargo clippy --workspace --all-targets -- -D warnings`
- **No unsafe code**: `#![forbid(unsafe_code)]`
- **No panics**: Deny `unwrap()` and `expect()` in production code
- **JSON naming**: `snake_case` via `#[serde(rename_all = "snake_case")]` — responses use snake_case JSON field naming (e.g., `user_id`, `tenant_id`)

### REST API Design

Follow the REST API guidelines: [`@/guidelines/DNA/REST/API.md`](../../../guidelines/DNA/REST/API.md)

**Key conventions:**
- **Resource URLs**: Nouns, plural (`/users`, `/types`)
- **Status codes**: See [`@/guidelines/DNA/REST/STATUS_CODES.md`](../../../guidelines/DNA/REST/STATUS_CODES.md)
- **Errors**: RFC-9457 Problem Details (`application/problem+json`)
- **Pagination**: Cursor-based with OData filtering (see [`@/guidelines/DNA/REST/PAGINATION.md`](../../../guidelines/DNA/REST/PAGINATION.md))
- **Timestamps**: ISO-8601 UTC with milliseconds (`2025-09-01T20:00:00.000Z`)

### Error Handling

- **Domain errors** → `src/domain/error.rs` (pure business errors)
- **SDK errors** → `<module>-sdk/src/errors.rs` (transport-agnostic)
- **REST mapping** → `impl From<DomainError> for Problem` in `src/api/rest/error.rs`
- **Handler return** → `ApiResult<T, DomainError>` with `ApiError::from_domain(e)`

### Security

Follow security guidelines: [`@/guidelines/SECURITY.md`](../../../guidelines/SECURITY.md)

**Key requirements:**
- **All API methods MUST accept `&SecurityContext`** as first parameter
- **Use SecureConn** for database access with automatic tenant isolation
- **Input validation** via `validator` crate
- **No secrets in code** — use environment variables

### Database Access

Follow secure ORM patterns: [`@/docs/modkit_unified_system/06_secure_orm_db_access.md`](../../../docs/modkit_unified_system/06_secure_orm_db_access.md)

- **Typestate enforcement**: Unscoped queries cannot execute
- **Deny-by-default**: Empty scopes return `WHERE 1=0`
- **Derive macro**: `#[derive(Scopable)]` with explicit dimension declarations
- **Request-scoped**: `SecurityContext` passed per-operation

### Testing Strategy

Target: **90%+ code coverage**

- **Unit tests**: `cargo test --workspace`
- **Integration tests**: `make test-pg` / `make test-sqlite`
- **E2E tests**: `make e2e-docker` (Python/pytest)
- **All checks**: `make check` or `python scripts/ci.py check`

### Observability

Follow tracing setup: [`@/docs/TRACING_SETUP.md`](../../../docs/TRACING_SETUP.md)

- **Structured logging** via `tracing` with contextual fields
- **Distributed tracing** with OpenTelemetry and W3C Trace Context
- **TracedClient** for instrumented HTTP calls

## Important Constraints

1. **No breaking changes** without version bump and deprecation period
2. **All dependencies** specified in root `Cargo.toml` (workspace inheritance)
3. **Feature flags** for optional functionality
4. **Backward compatibility**: Clients must ignore unknown fields

## Reference Documents

| Document | Purpose |
|----------|---------|
| [`@/docs/ARCHITECTURE_MANIFEST.md`](../../../docs/ARCHITECTURE_MANIFEST.md) | System architecture and design principles |
| [`@/docs/modkit_unified_system/README.md`](../../../docs/modkit_unified_system/README.md) | ModKit framework guide |
| [`@/docs/modkit_unified_system/06_secure_orm_db_access.md`](../../../docs/modkit_unified_system/06_secure_orm_db_access.md) | Secure database access patterns |
| [`@/docs/TRACING_SETUP.md`](../../../docs/TRACING_SETUP.md) | Observability configuration |
| [`@/guidelines/NEW_MODULE.md`](../../../guidelines/NEW_MODULE.md) | Step-by-step module creation |
| [`@/guidelines/SECURITY.md`](../../../guidelines/SECURITY.md) | Security best practices |
| [`@/guidelines/DNA/REST/API.md`](../../../guidelines/DNA/REST/API.md) | REST API design |
| [`@/guidelines/DNA/REST/STATUS_CODES.md`](../../../guidelines/DNA/REST/STATUS_CODES.md) | HTTP status code usage |
| [`@/guidelines/DNA/REST/PAGINATION.md`](../../../guidelines/DNA/REST/PAGINATION.md) | Cursor pagination spec |
| [`@/guidelines/DNA/languages/RUST.md`](../../../guidelines/DNA/languages/RUST.md) | Rust coding standards |
