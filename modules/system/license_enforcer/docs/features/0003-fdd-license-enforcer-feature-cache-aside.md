# Feature: Cache-Aside Resolution of Tenant Feature Sets

## 1. Feature Context

**ID**: `fdd-license-enforcer-feature-cache-aside`

**Status**: NON IMPLEMENTED

### 1.1 Overview

Resolve tenant licensing decisions using a cache-aside flow over a tenant’s enabled global feature set:
- read from cache plugin
- on miss (or cache failure), fetch from platform plugin
- store the result back in cache

### 1.2 Purpose

Minimize latency and Platform load while maintaining best-effort freshness for feature-gating decisions.

### 1.3 Actors

- `fdd-license-enforcer-actor-hs-module` - Requests feature checks
- `fdd-license-enforcer-actor-plugin` - Provides cache and platform integrations

### 1.4 References

- Overall Design: [DESIGN.md](../DESIGN.md)
- Dependencies: `fdd-license-enforcer-feature-plugin-discovery`

## 2. Actor Flows (FDL)

### Resolve Enabled Features (Cache-Aside)

- [ ] `ph-1` **ID**: `fdd-license-enforcer-feature-cache-aside-flow-resolve-enabled-features`

<!-- fdd-id-content -->
**Actor**: `fdd-license-enforcer-actor-hs-module`

**Success Scenarios**:
- Cache hit returns enabled features without a platform call.
- Cache miss triggers a platform call and caches the resulting feature set.

**Error Scenarios**:
- Missing tenant scope returns `MissingTenantScope` and performs no plugin calls.

**Steps**:
1. [ ] - `ph-1` - Module calls `enabled_global_features(ctx)` or `is_global_feature_enabled(ctx, feature_id)`. - `inst-cache-aside-1`
2. [ ] - `ph-1` - **IF** tenant scope missing in `SecurityContext` - `inst-cache-aside-2`
   1. [ ] - `ph-1` - **RETURN** `LicenseEnforcerError::MissingTenantScope`. - `inst-cache-aside-2a`
3. [ ] - `ph-1` - Gateway resolves cache plugin client (scoped by instance ID). - `inst-cache-aside-3`
4. [ ] - `ph-1` - Gateway calls `CachePluginClient::get_tenant_features(ctx)`. - `inst-cache-aside-4`
5. [ ] - `ph-1` - **IF** cache returns `Some(features)` - `inst-cache-aside-5`
   1. [ ] - `ph-1` - **RETURN** `features`. - `inst-cache-aside-5a`
6. [ ] - `ph-1` - **ELSE** cache miss or cache unavailable - `inst-cache-aside-6`
7. [ ] - `ph-1` - Gateway resolves platform plugin client (scoped by instance ID). - `inst-cache-aside-7`
8. [ ] - `ph-1` - Gateway calls `PlatformPluginClient::get_enabled_global_features(ctx)`. - `inst-cache-aside-8`
9. [ ] - `ph-1` - Gateway calls `CachePluginClient::set_tenant_features(ctx, features)` (best-effort). - `inst-cache-aside-9`
10. [ ] - `ph-1` - **RETURN** `features`. - `inst-cache-aside-10`
<!-- fdd-id-content -->

## 3. Algorithms (FDL)

### Cache-Aside Feature Resolution

- [ ] **ID**: `fdd-license-enforcer-feature-cache-aside-algo-cache-aside`

<!-- fdd-id-content -->
**Input**: `SecurityContext`

**Output**: `EnabledGlobalFeatures`

**Steps**:
1. [ ] - `ph-1` - Validate tenant scope from `SecurityContext`. - `inst-cache-aside-algo-1`
2. [ ] - `ph-1` - Attempt `CachePluginClient::get_tenant_features`. - `inst-cache-aside-algo-2`
3. [ ] - `ph-1` - **IF** cache hit, return cached features. - `inst-cache-aside-algo-3`
4. [ ] - `ph-1` - **ELSE** call `PlatformPluginClient::get_enabled_global_features`. - `inst-cache-aside-algo-4`
5. [ ] - `ph-1` - Store result using `CachePluginClient::set_tenant_features` (ignore errors). - `inst-cache-aside-algo-5`
6. [ ] - `ph-1` - **RETURN** platform-derived features. - `inst-cache-aside-algo-6`
<!-- fdd-id-content -->

## 4. States (FDL)

No state machine in Phase 1. Cache entry lifetime is managed by the selected cache plugin (for example TTL-based expiration in the in-memory cache plugin).

## 5. Requirements

### Implement Cache-Aside Flow

- [ ] **ID**: `fdd-license-enforcer-feature-cache-aside-req-cache-aside`

<!-- fdd-id-content -->
**Status**: COMPLETED

**Description**: The gateway SHALL resolve tenant feature checks using a cache-aside flow over the tenant’s enabled global feature set:
- read the tenant feature set from the cache plugin
- on cache miss, fetch enabled global features from the platform plugin
- cache the result best-effort

**Implementation details**:
- Cache client trait: `CachePluginClient` (`get_tenant_features`, `set_tenant_features`)
- Platform client trait: `PlatformPluginClient` (`get_enabled_global_features`)

**Implements**:
- `fdd-license-enforcer-feature-cache-aside-flow-resolve-enabled-features`
- `fdd-license-enforcer-feature-cache-aside-algo-cache-aside`

**Phases**:
- [ ] `ph-1`: Cache-aside for global features
<!-- fdd-id-content -->

### Reject Missing Tenant Scope

- [ ] **ID**: `fdd-license-enforcer-feature-cache-aside-req-missing-tenant`

<!-- fdd-id-content -->
**Status**: COMPLETED

**Description**: The gateway SHALL return an explicit error when `SecurityContext` lacks tenant scope and MUST NOT call platform or cache plugins.

**Implementation details**:
- Error: `LicenseEnforcerError::MissingTenantScope`

**Implements**:
- `fdd-license-enforcer-feature-cache-aside-flow-resolve-enabled-features`

**Phases**:
- [ ] `ph-1`: Missing-tenant behavior
<!-- fdd-id-content -->
