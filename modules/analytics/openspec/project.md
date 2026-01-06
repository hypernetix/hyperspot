# Analytics Module - OpenSpec Project Configuration

**Module**: Analytics  
**Technology**: Rust + Axum + SeaORM  
**Architecture**: Modkit-based with SDK pattern

---

## Project Conventions

### Module Structure
- **SDK crate** (`analytics-sdk`): Pure domain contracts, GTS types, no serde
- **Main crate** (`analytics`): Domain logic, REST API, database, gRPC

### Code Standards
- **Language**: Rust edition 2021
- **Linting**: Clippy (pedantic + 100+ deny rules), dylint (architectural lints)
- **Formatting**: rustfmt with project config
- **Safety**: No unsafe code, forbidden by workspace lints

### Database
- **ORM**: SeaORM 1.1
- **Security**: All queries use `.secure(ctx)` for tenant isolation
- **Entities**: Must derive `Scopable` trait
- **Migrations**: SeaORM migrations in `migrations/`

### API Layer
- **Framework**: Axum 0.8
- **Auth**: JWT via SecurityCtx extraction
- **OpenAPI**: utoipa 5.3 for documentation
- **Endpoints**: `/api/v1/*` with version prefix (enforced by dylint)
- **Error Format**: RFC 7807 Problem Details via modkit-errors

### Domain Model
- **Type System**: GTS (Global Type System)
- **Location**: `gts/types/` for JSON schemas
- **Identifier Format**: `gts.ainetx.hyperspot.analytics.<type>.v<version>`
- **Validation**: All GTS identifiers must be lowercase with underscores

### Testing
- **Framework**: Rust built-in (`cargo test`)
- **Integration**: testcontainers for database tests
- **Coverage**: Target >80% for critical paths

### Build Commands
```bash
# Development
cargo build --package analytics-sdk --package analytics
cargo run --package analytics

# Testing
cargo test --package analytics --all-features
cargo test --package analytics-sdk

# Quality
cargo clippy --package analytics -- -D warnings
cargo fmt --check --package analytics
cd dylint_lints && cargo build --release
cargo dylint --workspace de0101_no_serde_in_contract de0103_no_http_types_in_contract
```

### Security Requirements
- **Tenant Isolation**: SecurityCtx required in all handlers
- **JWT Propagation**: Automatic tenant context in all external calls
- **Row-Level Security**: Database queries filtered by tenant_id
- **No Serde in SDK**: Contract layer must be transport-agnostic

### Observability
- **Tracing**: OpenTelemetry via tracing crate
- **Instrumentation**: `#[instrument]` on all public functions
- **Structured Logging**: JSON format for production
- **Metrics**: Prometheus-compatible via modkit

---

## Change Requirements

Each OpenSpec change must:
- [ ] Follow modkit SDK + module pattern
- [ ] Use GTS types for all domain models
- [ ] Apply SecurityCtx for tenant isolation
- [ ] Include database migrations if needed
- [ ] Update OpenAPI spec if REST endpoints added
- [ ] Add tests (unit + integration)
- [ ] Pass all lints (clippy + dylint)
- [ ] Update architecture/DESIGN.md if design changes

---

## References

- **Architecture**: `../architecture/DESIGN.md`
- **Features**: `../architecture/features/FEATURES.md`
- **GTS Types**: `../gts/types/`
- **Modkit Docs**: `/docs/MODKIT_UNIFIED_SYSTEM.md`
