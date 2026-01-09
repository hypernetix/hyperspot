# Design: User Settings Module

## Context

The settings module is being migrated from Go to Rust as part of HyperSpot's modular architecture initiative. The module provides per-user settings (theme, language) with automatic tenant and user isolation. This is a foundational capability that demonstrates ModKit patterns and Secure ORM usage.

**Constraints:**
- Must follow SDK pattern (separate SDK crate for public API)
- Must use SecurityContext for all data access (no explicit tenant_id/user_id in APIs)
- Must implement Secure ORM with automatic tenant/user scoping
- Must follow DDD-light structure (api/domain/infra layers)
- Must provide REST API compatible with ModKit/api_gateway
- Database schema uses composite primary key (tenant_id, user_id)

**Stakeholders:**
- Module consumers via ClientHub (inter-module communication)
- REST API consumers (frontend, external clients)
- Development team (reference implementation for future modules)

## Goals / Non-Goals

**Goals:**
- Provide user-scoped settings storage with automatic tenant isolation
- Implement GET, POST (full update), PATCH (partial update) endpoints
- Follow ModKit conventions for type-safe REST and ClientHub integration
- Demonstrate Secure ORM patterns with SeaORM
- Support default/empty values for first-time users without database record
- Achieve 90%+ test coverage including security isolation tests

**Non-Goals:**
- Tenant-level or global settings (only user-scoped)
- Settings versioning or history tracking
- Complex validation rules (simple string storage)
- Real-time settings synchronization
- Settings import/export functionality
- Migration from Go implementation (manual data migration if needed)

## Decisions

### Decision 1: SDK Pattern with SecurityContext
**What:** Separate `settings-sdk` crate containing API trait, models, and errors. All API methods take `&SecurityContext` as first parameter.

**Why:**
- Clean separation of public API from implementation
- Transport-agnostic contracts (no serde in SDK)
- SecurityContext enables automatic tenant/user isolation
- Consumers only depend on lightweight SDK crate

**Alternatives considered:**
- Single crate without SDK: Rejected - violates ModKit SDK pattern
- Explicit tenant_id/user_id in API: Rejected - insecure, violates Secure ORM principles

### Decision 2: Composite Primary Key (tenant_id, user_id)
**What:** Database table uses composite primary key of (tenant_id, user_id) with no separate ID field.

**Why:**
- Natural key for user settings (one settings record per user-tenant pair)
- Aligns with Go implementation schema
- Enforces uniqueness at database level
- Simplifies queries (no joins needed)

**Alternatives considered:**
- Separate UUID primary key + unique constraint: Rejected - adds unnecessary complexity
- Tenant_id only: Rejected - doesn't support multi-user tenants

### Decision 3: Lazy Creation on Update
**What:** GET returns empty defaults if no record exists. Record is created only on POST/PATCH.

**Why:**
- Avoids database writes for users who never change settings
- Matches Go implementation behavior
- Simpler first-time user flow

**Alternatives considered:**
- Create on first GET: Rejected - unnecessary database writes
- Require explicit creation endpoint: Rejected - less ergonomic

### Decision 4: POST vs PATCH Semantics
**What:**
- POST: Full update, requires all fields (theme, language)
- PATCH: Partial update, accepts any subset of fields

**Why:**
- Follows REST conventions
- POST is simpler for form-based full settings update
- PATCH enables granular updates (e.g., only theme)

**Alternatives considered:**
- Only PUT/PATCH: Rejected - Go implementation has POST, maintain compatibility
- Only PATCH: Rejected - less explicit for full replacement

### Decision 5: Secure ORM with Scopable Derive
**What:** Use `#[derive(Scopable)]` on entity with automatic SecurityContext scoping via `SecureConn`.

**Why:**
- Compile-time safety (unscoped queries cannot execute)
- Automatic tenant/user filtering on all queries
- Deny-by-default security (empty scope returns WHERE 1=0)
- Consistent with HyperSpot security model

**Alternatives considered:**
- Manual WHERE clause addition: Rejected - error-prone, not compile-time safe
- Middleware-based filtering: Rejected - less explicit, harder to verify

## Risks / Trade-offs

### Risk: Composite Key Complexity with SeaORM
**Mitigation:** Follow users_info example which uses SeaORM with composite keys. Test thoroughly.

### Risk: Lazy Creation Race Condition
**Issue:** Multiple concurrent PATCH requests for new user might attempt multiple creates.

**Mitigation:** Use database-level UPSERT (INSERT ... ON CONFLICT UPDATE) or handle duplicate key errors gracefully. SeaORM's `save()` method handles this.

### Trade-off: No Settings History
**Decision:** Current design doesn't track settings changes over time.

**Reasoning:** Simplifies implementation, matches Go behavior. Can be added later if needed via event sourcing or audit log.

## Migration Plan

### Phase 1: Implementation (Rust module creation)
1. Create SDK crate with API trait, models, errors
2. Implement domain layer (service, repository trait)
3. Implement infrastructure (SeaORM entity, repo, migrations)
4. Implement REST API (handlers, routes, DTOs)
5. Write comprehensive tests

### Phase 2: Integration
1. Add module to hyperspot-server
2. Verify OpenAPI documentation
3. Test REST endpoints via api_gateway
4. Validate ClientHub integration

### Phase 3: Data Migration (if needed)
1. If existing Go data needs migration, create separate migration script
2. Verify tenant_id and user_id mapping
3. Test with sample data before production

### Rollback Strategy
- Module can be disabled by removing from hyperspot-server dependencies
- Database table can be dropped if needed (user settings are non-critical)
- No impact on other modules (settings is leaf dependency)

## Open Questions

1. **String length limits for theme/language?**
   - **Answer:** Use reasonable limits (e.g., 100 chars) with validation in domain layer

2. **Should settings support arbitrary JSON for extensibility?**
   - **Decision:** No, keep it simple with two string fields. Can be extended later if needed.

3. **Should we provide a "reset to defaults" endpoint?**
   - **Decision:** Not in initial implementation. Client can POST/PATCH with empty strings.

4. **How to handle settings for deleted users?**
   - **Decision:** Settings remain in database. Can add cleanup job later if needed.
