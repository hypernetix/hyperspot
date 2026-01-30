# ADR-0004: Use a Swappable Cache Plugin

**Date**: 2026-01-28

**Status**: Accepted

**ID**: `fdd-license-enforcer-adr-cache-plugin`

## Context and Problem Statement

Feature gating requires caching of tenant feature sets, but deployments vary in scale and infrastructure. We need a cache approach that supports both in-process and external backends.

## Decision Drivers

* Support in-memory and Redis-backed caches
* Keep the gateway independent of cache implementation details
* Allow deployments to choose the right cache for scale

## Considered Options

* Hardcode an in-memory cache in the gateway
* Hardcode Redis as the cache backend
* Delegate caching to a swappable cache plugin

## Decision Outcome

Chosen option: "Delegate caching to a swappable cache plugin", because it preserves flexibility and avoids locking the gateway to a single backend.

### Consequences

* Good, because deployments can choose in-memory or Redis
* Good, because cache logic is replaceable without changing the gateway
* Bad, because cache plugins add an integration surface to maintain

## Related Design Elements

**Requirements**:
* `fdd-license-enforcer-fr-cache-tenant-features` - Cache feature sets
* `fdd-license-enforcer-nfr-best-effort-freshness` - TTL-based freshness
