# ADR-0002: Enforce Tenant-Scoped Feature Checks

**Date**: 2026-01-28

**Status**: Accepted

**ID**: `fdd-license-enforcer-adr-tenant-scope`

## Context and Problem Statement

HyperSpot is multi-tenant, and license decisions must never leak across tenants. License checks must always use the tenant identifier supplied by `SecurityContext` to avoid cross-tenant access and cache pollution.

## Decision Drivers

* Strong tenant isolation and security
* Consistent behavior across modules
* Predictable cache semantics per tenant
* Avoid implicit or default tenant selection

## Considered Options

* Allow a default tenant when context is missing
* Require explicit tenant scope from `SecurityContext`
* Use a shared global feature cache across tenants

## Decision Outcome

Chosen option: "Require explicit tenant scope from `SecurityContext`", because it guarantees isolation and makes all feature checks deterministic and auditable per tenant. If tenant scope is missing, the gateway returns an explicit error (e.g., `MissingTenant`) and treats the request as denied.

### Consequences

* Good, because feature checks are isolated and safe
* Good, because missing-tenant requests fail fast and are auditable
* Bad, because modules must always provide a valid tenant context

## Related Design Elements

**Requirements**:
* `fdd-license-enforcer-nfr-tenant-isolation` - Tenant isolation constraint
* `fdd-license-enforcer-fr-check-global-feature` - Tenant-scoped checks
