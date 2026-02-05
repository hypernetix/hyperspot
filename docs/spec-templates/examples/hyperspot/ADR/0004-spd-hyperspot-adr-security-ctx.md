# ADR-0004: Security by Construction via SecurityCtx Propagation

**Date**: 2024-01-16

**Status**: Accepted

**ID**: `spd-hyperspot-adr-security-ctx`

## Context and Problem Statement

Multi-tenant SaaS platforms face critical risk of tenant data leakage through missing or incorrect tenant_id filters in database queries. Traditional approaches rely on runtime checks, ORM conventions, or developer discipline. We need compile-time guarantees that tenant isolation cannot be bypassed, even accidentally.

## Decision Drivers

* Must make it impossible to query database without tenant context at compile time
* Must propagate tenant context across module boundaries automatically
* Must work with Rust's type system to enforce security invariants
* Must not impose runtime overhead for context passing
* Must prevent SQL injection of tenant_id through type safety

## Considered Options

* Request-scoped SecurityCtx passed explicitly to all functions (chosen)
* Thread-local context set once per request (implicit propagation)
* ORM-level row-level security with automatic tenant_id injection
* Database-level RLS (Row-Level Security) policies

## Decision Outcome

Chosen option: "Request-scoped SecurityCtx passed explicitly to all functions", because Rust's type system can enforce that database query functions require SecurityCtx as a parameter. This makes tenant isolation violations a compilation error. The compiler verifies that every code path calling the database has obtained a valid SecurityCtx from the request.

### Consequences

* Good, because missing tenant context is a compilation error, not runtime bug
* Good, because SecurityCtx cannot be forged (opaque type with controlled construction)
* Good, because context propagation is visible in function signatures (auditable)
* Good, because zero runtime overhead (SecurityCtx is a small stack value)
* Good, because type system prevents accidentally using wrong tenant's context
* Bad, because requires SecurityCtx parameter in many function signatures (boilerplate)
* Bad, because refactoring functions to add database access requires propagating SecurityCtx
* Bad, because developers must explicitly pass context (cannot "forget" and rely on runtime checks)

## Related Design Elements

**Actors**:
* `spd-hyperspot-actor-tenant-admin` - Benefits from guaranteed tenant isolation
* `spd-hyperspot-actor-end-user` - Protected from cross-tenant data leaks
* `spd-hyperspot-actor-database-manager` - Enforces tenant context in queries

**Requirements**:
* `spd-hyperspot-fr-tenant-isolation` - Core requirement for SecurityCtx design
* `spd-hyperspot-nfr-tenant-security` - Compile-time guarantee of isolation
* `spd-hyperspot-fr-access-control` - SecurityCtx contains user identity for RBAC
