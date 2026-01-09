# Project Context

## Purpose

HyperSpot Server is a modular, high-performance platform for building AI services in Rust. Built on the **ModKit** framework, it provides:

- **Modular architecture** — Everything is a Module with composable, independent units
- **Type safety** — Compile-time guarantees via typestate builders and trait-based APIs
- **Multi-tenancy** — Built-in tenant isolation with secure ORM layer
- **Database agnostic** — PostgreSQL, MySQL, SQLite via unified API
- **GTS extensibility** — Global Type System for versioned, pluggable extensions

## Tech Stack

- **Language**: Rust (see [`@/guidelines/DNA/languages/RUST.md`](../guidelines/DNA/languages/RUST.md))
- **Framework**: ModKit (see [`@/docs/MODKIT_UNIFIED_SYSTEM.md`](../docs/MODKIT_UNIFIED_SYSTEM.md))
- **HTTP**: Axum with `tower-http` middleware
- **gRPC**: Tonic (out-of-process modules)
- **Database**: SeaORM / SQLx with secure ORM layer (see [`@/docs/SECURE-ORM.md`](../docs/SECURE-ORM.md))
- **Observability**: `tracing` + OpenTelemetry (see [`@/docs/TRACING_SETUP.md`](../docs/TRACING_SETUP.md))
- **OpenAPI**: `utoipa` for automatic documentation generation
- **Testing**: cargo test, pytest (E2E), testcontainers
- **Linting**: Clippy (pedantic), custom dylint linters

## Project Conventions

### Architecture

Follow the architecture manifest: [`@/docs/ARCHITECTURE_MANIFEST.md`](../docs/ARCHITECTURE_MANIFEST.md)

**Key principles:**
- **Everything is a Module** — composable, independent units
- **DDD-light structure** — API / Contract / Domain / Infra layers
- **SDK pattern** — separate `-sdk` crate for public API surface
- **Type-safe REST** — OperationBuilder prevents half-wired routes at compile time

### Module Structure

Follow the new module guideline: [`@/guidelines/NEW_MODULE.md`](../guidelines/NEW_MODULE.md)

```
modules/<module>/
├─ <module>-sdk/           # Public API: trait, models, errors (NO serde)
│  └─ src/
│     ├─ api.rs            # API trait (all methods take &SecurityCtx)
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

See [`@/guidelines/DNA/languages/RUST.md`](../guidelines/DNA/languages/RUST.md)

- **Formatting**: `cargo fmt` with max line length 100, 4-space indentation
- **Linting**: `cargo clippy --workspace --all-targets -- -D warnings`
- **Architecture lints**: Custom dylint linters enforce layer separation
- **No unsafe code**: `#![forbid(unsafe_code)]`
- **No panics**: Deny `unwrap()` and `expect()` in production code
- **JSON naming**: `camelCase` via `#[serde(rename_all = "camelCase")]`

### REST API Design

Follow the REST API guidelines: [`@/guidelines/DNA/REST/API.md`](../guidelines/DNA/REST/API.md)

**Key conventions:**
- **Endpoint format**: `/{service-name}/v{N}/{resource}` (versioning required)
- **Status codes**: See [`@/guidelines/DNA/REST/STATUS_CODES.md`](../guidelines/DNA/REST/STATUS_CODES.md)
- **Errors**: RFC-9457 Problem Details (`application/problem+json`)
- **Pagination**: Cursor-based with OData filtering (see [`@/guidelines/DNA/REST/PAGINATION.md`](../guidelines/DNA/REST/PAGINATION.md))

### Error Handling

- **Domain errors** → `src/domain/error.rs` (pure business errors)
- **SDK errors** → `<module>-sdk/src/errors.rs` (transport-agnostic)
- **REST mapping** → `impl From<DomainError> for Problem` in `src/api/rest/error.rs`
- **Handler return** → `ApiResult<T, DomainError>` with `ApiError::from_domain(e)`

### Security

Follow security guidelines: [`@/guidelines/SECURITY.md`](../guidelines/SECURITY.md)

**Key requirements:**
- **All API methods MUST accept `&SecurityCtx`** as first parameter
- **Use SecureConn** for database access with automatic tenant isolation
- **Input validation** via `validator` crate
- **No secrets in code** — use environment variables

### Database Access

Follow secure ORM patterns: [`@/docs/SECURE-ORM.md`](../docs/SECURE-ORM.md)

- **Typestate enforcement**: Unscoped queries cannot execute
- **Deny-by-default**: Empty scopes return `WHERE 1=0`
- **Request-scoped**: `SecurityCtx` passed per-operation

### Testing Strategy

Target: **90%+ code coverage**

- **Unit tests**: `cargo test --workspace`
- **Integration tests**: `make test-pg` / `make test-sqlite`
- **E2E tests**: `make e2e-docker` (Python/pytest)
- **All checks**: `make check` or `make ci`

### Git Workflow

**Commit format:** `<type>(<scope>): <description>`

Types: `feat`, `fix`, `tech`, `cleanup`, `refactor`, `test`, `docs`, `style`, `chore`, `perf`, `ci`, `build`, `revert`, `security`, `breaking`

**DCO required:** All commits must be signed off (`git commit -s`)

## Important Constraints

### Linter-Enforced Rules (dylint)
- **DE01xx**: Contract layer must be pure — NO serde, NO utoipa, NO HTTP types
- **DE02xx**: DTOs must be in `api/rest/` with serde + utoipa derives
- **DE08xx**: REST endpoints MUST be versioned (`/service/v1/resource`)

### Breaking Changes
- No breaking changes without version bump and deprecation period
- All dependencies specified in root `Cargo.toml` (workspace inheritance)
- Backward compatibility: Clients must ignore unknown fields

## Module-Specific OpenSpec

Each module in `modules/` can have its own `openspec/` directory for module-specific specs and changes. This enables domain teams to own their module's specifications.

**Example**: See [`modules/types-registry/openspec/`](../modules/types-registry/openspec/) for a complete example:

```
modules/types-registry/openspec/
├── project.md              # Module-specific context (references parent guidelines)
├── AGENTS.md               # AI assistant instructions
├── specs/                  # Current module specifications
│   ├── types-registry/
│   │   └── spec.md
│   └── types-registry-sdk/
│       └── spec.md
└── changes/                # Module change proposals
    └── archive/            # Completed changes
```

**Root vs Module OpenSpec:**
- **Root (`openspec/`)**: Cross-cutting concerns, shared infrastructure, project-wide changes
- **Module (`modules/<name>/openspec/`)**: Module-specific features, APIs, and implementations

## Reference Documents

| Document | Purpose |
|----------|---------|
| [`@/docs/ARCHITECTURE_MANIFEST.md`](../docs/ARCHITECTURE_MANIFEST.md) | System architecture and design principles |
| [`@/docs/MODKIT_UNIFIED_SYSTEM.md`](../docs/MODKIT_UNIFIED_SYSTEM.md) | ModKit framework guide |
| [`@/docs/SECURE-ORM.md`](../docs/SECURE-ORM.md) | Secure database access patterns |
| [`@/docs/TRACING_SETUP.md`](../docs/TRACING_SETUP.md) | Observability configuration |
| [`@/guidelines/NEW_MODULE.md`](../guidelines/NEW_MODULE.md) | Step-by-step module creation |
| [`@/guidelines/SECURITY.md`](../guidelines/SECURITY.md) | Security best practices |
| [`@/guidelines/DNA/REST/API.md`](../guidelines/DNA/REST/API.md) | REST API design |
| [`@/guidelines/DNA/REST/STATUS_CODES.md`](../guidelines/DNA/REST/STATUS_CODES.md) | HTTP status code usage |
| [`@/guidelines/DNA/REST/PAGINATION.md`](../guidelines/DNA/REST/PAGINATION.md) | Cursor pagination spec |
| [`@/guidelines/DNA/languages/RUST.md`](../guidelines/DNA/languages/RUST.md) | Rust coding standards |
