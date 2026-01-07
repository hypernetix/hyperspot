# Code & Documentation Conventions

**Language**: Rust (edition 2021)
**Style**: Standard Rust conventions + project-specific rules

---

## Code Conventions

### Naming
- Types: PascalCase (e.g., `UserProfile`)
- Functions: snake_case (e.g., `create_user`)
- Constants: SCREAMING_SNAKE_CASE (e.g., `MAX_RETRIES`)
- GTS identifiers: all lowercase with underscores (e.g., `user_profile`)

### Safety
- No unsafe code allowed (forbidden by clippy rules)
- 100+ clippy deny rules enforced
- Custom architectural lints via dylint

### Layer Separation
- Contract layer: Pure domain types, no HTTP/serde dependencies
- Domain layer: Business logic only
- API layer: DTOs with serde, HTTP types allowed
- Infra layer: External dependencies isolated

---

## Platform Helpers

**⚠️ CRITICAL**: Always check if platform helpers can be reused instead of reimplementing.

### DO NOT Reimplement
- ❌ GTS identifier parsing → use `GtsID::new()`
- ❌ JWT validation → use `AuthDispatcher`
- ❌ Problem Details errors → use `Problem`, `bad_request()`
- ❌ OData pagination → use `Page`, `PageInfo`
- ❌ Security context → use `SecurityCtx`
- ❌ OpenAPI generation → use `OperationBuilder`

### Helper Selection
1. **GTS needs** → Use `gts` crate (`GtsID::new()`, `GtsOps`)
2. **API needs** → Use `modkit::OperationBuilder`, `Problem`
3. **Auth needs** → Use `modkit-auth::AuthDispatcher`, `Claims`
4. **OData needs** → Use `modkit-odata` for pagination, filtering
5. **Security needs** → Use `modkit-security::SecurityCtx`, `Permission`
6. **DB needs** → Use `modkit-db::DbPool`, OData query builder

---

## Documentation

- Use Rust doc comments (`///`, `//!`)
- OpenAPI docs via utoipa annotations
- Architecture docs in FDD format

---

## Error Handling

- Use `Problem` for RFC 7807 Problem Details
- Domain errors in Result types
- Never panic in production code

---

## Non-Functional Requirements

- **Performance**: API response < 200ms
- **Observability**: Structured logging, distributed tracing
- **Modularity**: Plugin-based, hot-reloadable
- **Scalability**: Stateless, horizontal scaling ready
- **Type Safety**: GTS-based cross-module validation
