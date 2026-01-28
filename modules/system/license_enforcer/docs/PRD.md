# PRD

## 1. Overview

**Purpose**: The License Enforcer module is HyperSpot’s centralized feature-gating service that asks the Platform which features are enabled for a tenant and exposes a stable SDK to the rest of the system.

License enforcement must be consistent across all modules without embedding Platform-specific licensing logic in each module. The License Enforcer achieves this by providing a common client API for feature checks and delegating Platform calls to configurable plugins.

Phase 1 focuses on tenant-scoped, read-only checks for **global features** (tenant-wide feature set). It does not implement subscriptions, per-user/per-resource licensing, quotas, or usage billing; those concerns are owned by the Platform and the usage module.

**HyperSpot module developers** - Integrate feature checks without duplicating Platform-specific licensing logic.
**System operators** - Configure the active licensing plugin and feature ID mappings for their Platform.

**Key Problems Solved**:
- **Inconsistent feature access**: A single tenant-scoped gate ensures all modules enforce the same licensing decisions.
- **Platform coupling**: Plugin-based integration isolates Platform APIs and feature ID mapping from core modules.

**Success Criteria**:
- Feature checks always resolve using the tenant scope from `SecurityContext` and return only that tenant’s enabled global features.
- Tenant feature caching uses a swappable cache plugin (in-memory, Redis, etc.), respects a configurable TTL, and refreshes on cache misses.
- Platform feature identifiers can be mapped to HyperSpot `LicenseFeatureID`s through plugins.

**Capabilities**:
- Tenant-scoped `is_global_feature_enabled` checks
- Fetching the full set of enabled global features for a tenant
- Tenant feature caching via swappable cache plugin with configurable TTL
- Plugin-based Platform integration and feature ID mapping

## 2. Actors

### 2.1 Human Actors

#### System Operator

**ID**: `fdd-license-enforcer-actor-system-operator`

<!-- fdd-id-content -->
**Role**: Configures the licensing plugin and feature ID mappings for the deployment and monitors that feature gating aligns with purchased offerings.
<!-- fdd-id-content -->

#### Tenant Administrator

**ID**: `fdd-license-enforcer-actor-tenant-admin`

<!-- fdd-id-content -->
**Role**: Enables or purchases services on the Platform and expects HyperSpot to enforce feature access accordingly.
<!-- fdd-id-content -->

### 2.2 System Actors

#### HyperSpot Module

**ID**: `fdd-license-enforcer-actor-hs-module`

<!-- fdd-id-content -->
**Role**: Calls the License Enforcer SDK to determine whether a tenant can access a feature.
<!-- fdd-id-content -->

#### License Enforcement Plugin

**ID**: `fdd-license-enforcer-actor-plugin`

<!-- fdd-id-content -->
**Role**: Integrates with the Platform licensing API and maps Platform feature identifiers to HyperSpot feature IDs.
<!-- fdd-id-content -->

#### Platform Licensing API

**ID**: `fdd-license-enforcer-actor-platform-api`

<!-- fdd-id-content -->
**Role**: Source of truth for which features are enabled for a tenant in the Platform.
<!-- fdd-id-content -->

## 3. Functional Requirements

#### Check Global Feature Enablement

**ID**: `fdd-license-enforcer-fr-check-global-feature`

<!-- fdd-id-content -->
**Priority**: Critical

The system must return whether a specified global feature is enabled for the tenant defined in `SecurityContext`.

**Actors**: `fdd-license-enforcer-actor-hs-module`, `fdd-license-enforcer-actor-plugin`
<!-- fdd-id-content -->

#### List Enabled Global Features

**ID**: `fdd-license-enforcer-fr-list-enabled-features`

<!-- fdd-id-content -->
**Priority**: High

The system must return the complete set of enabled global features for a tenant without pagination.

**Actors**: `fdd-license-enforcer-actor-hs-module`, `fdd-license-enforcer-actor-plugin`
<!-- fdd-id-content -->

#### Cache Tenant Feature Sets

**ID**: `fdd-license-enforcer-fr-cache-tenant-features`

<!-- fdd-id-content -->
**Priority**: High

The system must cache feature sets per tenant using a swappable cache plugin (e.g., in-memory, Redis), a configurable TTL, and refresh from the Platform on cache miss.

**Actors**: `fdd-license-enforcer-actor-hs-module`, `fdd-license-enforcer-actor-plugin`
<!-- fdd-id-content -->

#### Support Plugin-Based Platform Integration

**ID**: `fdd-license-enforcer-fr-plugin-integration`

<!-- fdd-id-content -->
**Priority**: High

The system must delegate Platform-specific feature retrieval and feature ID mapping to plugins so the core module remains Platform-agnostic.

**Actors**: `fdd-license-enforcer-actor-system-operator`, `fdd-license-enforcer-actor-plugin`
<!-- fdd-id-content -->

#### Map Platform Feature IDs to HyperSpot IDs

**ID**: `fdd-license-enforcer-fr-feature-id-mapping`

<!-- fdd-id-content -->
**Priority**: Medium

The system must support mapping Platform feature identifiers (for example, Cyber Workspace features such as `cti.a.p.lic.feature.v1.0~a.cyber_chat.v1.0`) to HyperSpot `LicenseFeatureID` values used by modules.

**Actors**: `fdd-license-enforcer-actor-system-operator`, `fdd-license-enforcer-actor-plugin`
<!-- fdd-id-content -->

## 4. Use Cases

#### UC-001: Check a Feature for a Tenant

**ID**: `fdd-license-enforcer-usecase-check-feature`

<!-- fdd-id-content -->
**Actor**: `fdd-license-enforcer-actor-hs-module`

**Preconditions**: Module has a `SecurityContext` with tenant scope and a `LicenseFeatureID` to verify.

**Flow**:
1. Module calls `is_global_feature_enabled(ctx, feature_id)`.
2. License Enforcer retrieves tenant features from cache.
3. License Enforcer returns whether the feature is present in the tenant’s feature set.

**Postconditions**: Module receives a deterministic allow/deny decision for the tenant.

**Acceptance criteria**:
- The check uses the tenant from `SecurityContext` and never a global or default tenant.
- The response is derived from the tenant’s enabled global feature set.

<!-- fdd-id-content -->

#### UC-002: Refresh Features on Cache Miss

**ID**: `fdd-license-enforcer-usecase-refresh-features`

<!-- fdd-id-content -->
**Actor**: `fdd-license-enforcer-actor-hs-module`

**Preconditions**: No cached feature set exists for the tenant, or the TTL has expired.

**Flow**:
1. Module calls `enabled_global_features(ctx)`.
2. License Enforcer asks the plugin for enabled global features from the Platform.
3. License Enforcer stores the feature set in cache with the configured TTL.
4. License Enforcer returns the feature set to the module.

**Postconditions**: Tenant feature set is cached and available for subsequent checks.

**Acceptance criteria**:
- The Platform is queried only when cache is missing or expired.
- The cached set is keyed by tenant ID.

<!-- fdd-id-content -->

#### UC-003: Configure Feature Mapping for a Platform Offering

**ID**: `fdd-license-enforcer-usecase-configure-mapping`

<!-- fdd-id-content -->
**Actor**: `fdd-license-enforcer-actor-system-operator`

**Preconditions**: The Platform defines features for an offering (for example, Cyber Workspace features for Cyber Chat users or Cyber Employee agents).

**Flow**:
1. Operator configures the active plugin and provides mappings between Platform feature IDs and HyperSpot `LicenseFeatureID`s.
2. Plugin uses the mapping when translating feature responses from the Platform.
3. Modules check features using HyperSpot IDs.

**Postconditions**: Feature checks in HyperSpot resolve correctly for the Platform offering.

**Acceptance criteria**:
- Platform feature IDs are translated consistently for all tenant checks.
- Modules do not need to reference Platform-specific identifiers directly.

<!-- fdd-id-content -->

## 5. Non-functional requirements

#### Tenant Isolation

**ID**: `fdd-license-enforcer-nfr-tenant-isolation`

<!-- fdd-id-content -->
Feature checks must be strictly tenant-scoped, ensuring data or feature entitlements from one tenant are never returned for another.
<!-- fdd-id-content -->

#### Best-Effort Freshness

**ID**: `fdd-license-enforcer-nfr-best-effort-freshness`

<!-- fdd-id-content -->
Cached feature sets must honor the configured TTL to balance freshness and Platform load.
<!-- fdd-id-content -->

#### Platform Agnosticism

**ID**: `fdd-license-enforcer-nfr-platform-agnostic`

<!-- fdd-id-content -->
Core License Enforcer logic must remain free of Platform-specific API contracts and rely on plugins for integration.
<!-- fdd-id-content -->

## 6. Additional context

#### Phase 1 Scope and Out-of-Scope Items

**ID**: `fdd-license-enforcer-prd-context-phase1-scope`

<!-- fdd-id-content -->
Phase 1 includes tenant-scoped global feature checks only. Per-user or per-resource licensing, quota enforcement, and usage billing remain out of scope and are handled by the Platform and the usage module.
<!-- fdd-id-content -->

#### Business Examples (Cyber Workspace)

**ID**: `fdd-license-enforcer-prd-context-cyber-workspace`

<!-- fdd-id-content -->
Cyber Workspace offerings introduce global features such as `cti.a.p.lic.feature.v1.0~a.cyber_chat.v1.0`, `cti.a.p.lic.feature.v1.0~a.cyber_employee.agents.v1.0`, and `cti.a.p.lic.feature.v1.0~a.cyber_employee.units.v1.0`. These are representative of the Platform feature identifiers that plugins must map to HyperSpot feature IDs for gating.
<!-- fdd-id-content -->
