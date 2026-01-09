# Tech Stack Specification

**Source**: Discovered from project structure, Cargo.toml, and architecture documents

---

## Languages

**Primary Language**: Rust  
**Edition**: 2021  
**Version**: Latest stable (1.70+)  

**Secondary Language**: Python  
**Version**: 3.11+  
**Purpose**: E2E testing, scripting

---

## Core Frameworks

**Web Framework**: Axum (tokio-based async)  
**Purpose**: HTTP server, REST API, middleware

**RPC Framework**: tonic  
**Purpose**: gRPC server and client implementation

**Async Runtime**: Tokio  
**Purpose**: Asynchronous runtime for all async operations

---

## Type System

**GTS (Global Type System)**  
**Specification**: `guidelines/GTS/`  
**Purpose**: Domain model validation, cross-module type safety  
**Format**: `gts.vendor.package.namespace.type.vMAJOR[.MINOR]`

**JSON Schema**: Draft-07  
**Purpose**: Schema validation, API contracts

---

## API Technologies

**REST API**:
- OpenAPI 3.x via utoipa macros
- Runtime endpoint: `/openapi.json`
- Validation: openapi-spec-validator

**gRPC**:
- Protocol Buffers (proto3)
- Code generation: tonic-build
- Location: `proto/`

---

## Database

**Type**: Relational (PostgreSQL compatible)  
**ORM**: Custom modkit-db with OData query support  
**Features**:
- OData `$select` field projection
- OData `$filter`, `$orderby`, `$top`, `$skip`
- Type-safe query builder
- Testcontainers for integration tests

---

## Authentication & Security

**JWT**: JSON Web Tokens via jsonwebtoken crate  
**Auth System**: modkit-auth with AuthDispatcher  
**Security Context**: modkit-security with SecurityCtx  
**Standards**: RFC 7807 Problem Details for errors

---

## Testing

**Unit Tests**: cargo test  
**Integration Tests**: cargo test --test '*' with testcontainers  
**E2E Tests**: pytest with httpx/playwright  
**Coverage**: cargo tarpaulin

---

## Build Tools

**Build System**: Cargo + Make  
**Linting**: 
- clippy (100+ deny rules)
- dylint (custom architectural lints)
- rustfmt

**CI/CD**: GitHub Actions (`.github/workflows/ci.yml`)

---

## Observability

**Tracing**: OpenTelemetry via tracing crate  
**Logging**: Structured logging with tracing  
**Metrics**: Custom metrics via modkit

---

## Development Tools

**Package Manager**: Cargo  
**Dependency Auditing**: cargo-deny  
**Unsafe Code Detection**: cargo-geiger  
**Custom Lints**: dylint framework with project-specific lints

---

## Module System

**Architecture**: modkit plugin system  

**Core Libraries**:
- **modkit**: Core framework (module lifecycle, REST API builder, client hub)
- **modkit-macros**: Procedural macros for module declaration
- **modkit-db**: Database abstractions (SeaORM, SQLx integration)
- **modkit-db-macros**: Database-related procedural macros
- **modkit-auth**: Authentication and JWT validation
- **modkit-security**: Security context and permissions
- **modkit-errors**: RFC 7807 Problem Details error handling
- **modkit-errors-macro**: Error generation macros
- **modkit-odata**: OData v4 query parsing (`$select`, `$filter`, `$orderby`)
- **modkit-node-info**: Node registry and discovery
- **modkit-transport-grpc**: gRPC transport layer abstractions

**Features**:
- Hot-reloadable modules
- Strict layer separation (contract/domain/api/infra)
- GTS-based type validation
- Dependency injection
- Type-safe inter-module communication
- Automatic OpenAPI documentation generation

---

## Version Requirements

```toml
[toolchain]
rust = "stable (1.70+)"
cargo = "latest"

[python]
version = "3.11+"
pytest = ">=7.0"
httpx = ">=0.24"

[validation]
openapi-spec-validator = ">=0.7.1"
```

---

## Source References

- Cargo.toml files across workspace
- dylint_lints/ for custom architectural rules
- docs/ARCHITECTURE_MANIFEST.md
- docs/MODKIT_UNIFIED_SYSTEM.md
- guidelines/GTS/README.md
