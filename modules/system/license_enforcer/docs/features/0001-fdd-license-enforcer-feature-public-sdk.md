# Feature: Public Global Feature SDK

## 1. Feature Context

**ID**: `fdd-license-enforcer-feature-public-sdk`

**Status**: NON IMPLEMENTED

### 1.1 Overview

A stable, tenant-scoped SDK that HyperSpot modules use to check whether a global feature is enabled for the tenant in `SecurityContext`, or to fetch the full enabled global feature set.

### 1.2 Purpose

Provide a single, consistent feature-gating API for all modules while keeping Platform licensing details out of module code.

### 1.3 Actors

- `fdd-license-enforcer-actor-hs-module` - Calls the SDK for allow/deny decisions

### 1.4 References

- Overall Design: [DESIGN.md](../DESIGN.md)
- Dependencies: `fdd-license-enforcer-feature-plugin-discovery`, `fdd-license-enforcer-feature-cache-aside`

## 2. Actor Flows (FDL)

### Check Global Feature Enablement

- [ ] `ph-1` **ID**: `fdd-license-enforcer-feature-public-sdk-flow-check-global-feature`

<!-- fdd-id-content -->
**Actor**: `fdd-license-enforcer-actor-hs-module`

**Success Scenarios**:
- The SDK returns `true` only if the requested `LicenseFeatureID` is present in the tenant's enabled global feature set.

**Error Scenarios**:
- If `SecurityContext` has no tenant scope, the SDK returns `MissingTenantScope` and performs no plugin calls.

**Steps**:
1. [ ] - `ph-1` - Module obtains `LicenseEnforcerGatewayClient` from `ClientHub` (unscoped). - `inst-public-sdk-check-1`
2. [ ] - `ph-1` - Module calls `is_global_feature_enabled(ctx, feature_id)`. - `inst-public-sdk-check-2`
3. [ ] - `ph-1` - **IF** tenant scope missing in `SecurityContext` - `inst-public-sdk-check-3`
   1. [ ] - `ph-1` - **RETURN** `LicenseEnforcerError::MissingTenantScope`. - `inst-public-sdk-check-3a`
4. [ ] - `ph-1` - **ELSE** gateway resolves enabled features (via cache-aside). - `inst-public-sdk-check-4`
5. [ ] - `ph-1` - **RETURN** `true|false` based on membership in enabled feature set. - `inst-public-sdk-check-5`
<!-- fdd-id-content -->

### List Enabled Global Features

- [ ] `ph-1` **ID**: `fdd-license-enforcer-feature-public-sdk-flow-list-enabled-features`

<!-- fdd-id-content -->
**Actor**: `fdd-license-enforcer-actor-hs-module`

**Success Scenarios**:
- The SDK returns the full set of enabled global features for the tenant (no pagination).

**Error Scenarios**:
- If `SecurityContext` has no tenant scope, the SDK returns `MissingTenantScope`.

**Steps**:
1. [ ] - `ph-1` - Module calls `enabled_global_features(ctx)`. - `inst-public-sdk-list-1`
2. [ ] - `ph-1` - Gateway returns the full `EnabledGlobalFeatures` set. - `inst-public-sdk-list-2`
<!-- fdd-id-content -->

## 3. Algorithms (FDL)

### Global Feature Constants Usage

- [ ] **ID**: `fdd-license-enforcer-feature-public-sdk-algo-global-feature-constants`

<!-- fdd-id-content -->
**Input**: SDK constant from `license_enforcer_sdk::global_features`.

**Output**: `LicenseFeatureID` to pass to the SDK APIs.

**Steps**:
1. [ ] - `ph-1` - Select a constant (e.g. `global_features::CYBER_CHAT`). - `inst-public-sdk-const-1`
2. [ ] - `ph-1` - Convert it via `global_features::to_feature_id(constant)`. - `inst-public-sdk-const-2`
3. [ ] - `ph-1` - **RETURN** `LicenseFeatureID`. - `inst-public-sdk-const-3`
<!-- fdd-id-content -->

## 4. States (FDL)

No explicit state machine in Phase 1. The returned decision is derived from the tenant's enabled global feature set.

## 5. Requirements

### Expose Public SDK Global Feature API

- [ ] **ID**: `fdd-license-enforcer-feature-public-sdk-req-sdk-api`

<!-- fdd-id-content -->
**Status**: COMPLETED

**Description**: The system SHALL expose a public SDK client (`LicenseEnforcerGatewayClient`) registered unscoped in `ClientHub` that allows modules to:
- call `is_global_feature_enabled(ctx, feature_id)`
- call `enabled_global_features(ctx)`

The tenant scope MUST be derived exclusively from `SecurityContext` and the SDK MUST NOT require an explicit tenant ID parameter.

**Implementation details**:
- API: `license_enforcer_sdk::LicenseEnforcerGatewayClient`
- Errors: `LicenseEnforcerError::MissingTenantScope` on missing tenant

**Implements**:
- `fdd-license-enforcer-feature-public-sdk-flow-check-global-feature`
- `fdd-license-enforcer-feature-public-sdk-flow-list-enabled-features`

**Phases**:
- [ ] `ph-1`: Global feature checks and listing
<!-- fdd-id-content -->

### Expose Convenience Global Feature Constants

- [ ] **ID**: `fdd-license-enforcer-feature-public-sdk-req-global-feature-constants`

<!-- fdd-id-content -->
**Status**: COMPLETED

**Description**: The SDK SHALL expose convenience constants for global license features and a helper for converting them into `LicenseFeatureID` values.

**Implementation details**:
- Constants in `license_enforcer_sdk::global_features`:
  - `BASE`
  - `CYBER_CHAT`
  - `CYBER_EMPLOYEE_AGENTS`
  - `CYBER_EMPLOYEE_UNITS`
- Helper: `global_features::to_feature_id(gts_id)`

**Implements**:
- `fdd-license-enforcer-feature-public-sdk-algo-global-feature-constants`

**Phases**:
- [ ] `ph-1`: Provide constants + conversion helper
<!-- fdd-id-content -->
