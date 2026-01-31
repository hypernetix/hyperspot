# Feature: Plugin Discovery and Wiring

## 1. Feature Context

**ID**: `fdd-license-enforcer-feature-plugin-discovery`

**Status**: NON IMPLEMENTED

### 1.1 Overview

Discovery and wiring of platform and cache plugins through `types-registry` and ModKit `ClientHub` scoped registration, using GTS plugin schema IDs and GTS instance IDs.

### 1.2 Purpose

Allow the gateway to remain Platform-agnostic while supporting multiple implementations per deployment and vendor-based selection.

### 1.3 Actors

- `fdd-license-enforcer-actor-system-operator` - Deploys/configures gateway and plugins
- `fdd-license-enforcer-actor-plugin` - Registers plugin instance and scoped client

### 1.4 References

- Overall Design: [DESIGN.md](../DESIGN.md)
- Dependencies: None

## 2. Actor Flows (FDL)

### Gateway Registers Plugin Schemas

- [ ] `ph-1` **ID**: `fdd-license-enforcer-feature-plugin-discovery-flow-gateway-register-schemas`

<!-- fdd-id-content -->
**Actor**: `fdd-license-enforcer-actor-system-operator`

**Success Scenarios**:
- Gateway registers platform and cache plugin schemas during initialization.

**Error Scenarios**:
- Types registry is unavailable and initialization fails.

**Steps**:
1. [ ] - `ph-1` - Gateway initializes and loads its configuration. - `inst-plugin-discovery-1`
2. [ ] - `ph-1` - Gateway registers platform plugin schema `LicensePlatformPluginSpecV1` in `types-registry`. - `inst-plugin-discovery-2`
3. [ ] - `ph-1` - Gateway registers cache plugin schema `LicenseCachePluginSpecV1` in `types-registry`. - `inst-plugin-discovery-3`
4. [ ] - `ph-1` - Gateway registers an unscoped `LicenseEnforcerGatewayClient` in `ClientHub`. - `inst-plugin-discovery-4`
<!-- fdd-id-content -->

### Plugin Registers Instance and Scoped Client

- [ ] `ph-1` **ID**: `fdd-license-enforcer-feature-plugin-discovery-flow-plugin-registers`

<!-- fdd-id-content -->
**Actor**: `fdd-license-enforcer-actor-plugin`

**Success Scenarios**:
- Plugin registers its instance metadata in `types-registry`.
- Plugin registers its client implementation in `ClientHub` under `ClientScope::gts_id(instance_id)`.

**Error Scenarios**:
- Plugin fails to register instance metadata.

**Steps**:
1. [ ] - `ph-1` - Plugin generates its GTS instance ID based on its plugin schema. - `inst-plugin-discovery-plugin-1`
2. [ ] - `ph-1` - Plugin registers a `BaseModkitPluginV1<...>` entity in `types-registry`. - `inst-plugin-discovery-plugin-2`
3. [ ] - `ph-1` - Plugin registers a scoped client in `ClientHub` using `ClientScope::gts_id(instance_id)`. - `inst-plugin-discovery-plugin-3`
<!-- fdd-id-content -->

## 3. Algorithms (FDL)

### Choose Plugin Instance

- [ ] **ID**: `fdd-license-enforcer-feature-plugin-discovery-algo-choose-plugin-instance`

<!-- fdd-id-content -->
**Input**: Vendor name, list of plugin instances from `types-registry`.

**Output**: Selected GTS instance ID.

**Steps**:
1. [ ] - `ph-1` - List instances from `types-registry` matching schema ID prefix. - `inst-plugin-discovery-algo-1`
2. [ ] - `ph-1` - Deserialize each instance as `BaseModkitPluginV1<Spec>`. - `inst-plugin-discovery-algo-2`
3. [ ] - `ph-1` - Filter by `vendor` matching gateway vendor. - `inst-plugin-discovery-algo-3`
4. [ ] - `ph-1` - Select the instance with the lowest `priority` value. - `inst-plugin-discovery-algo-4`
5. [ ] - `ph-1` - **RETURN** selected instance ID. - `inst-plugin-discovery-algo-5`
<!-- fdd-id-content -->

## 4. States (FDL)

No explicit state machine. Plugin resolution is lazy and cached in-process by the gateway after the first successful selection.

## 5. Requirements

### Register Plugin Schemas

- [ ] **ID**: `fdd-license-enforcer-feature-plugin-discovery-req-schema-registration`

<!-- fdd-id-content -->
**Status**: COMPLETED

**Description**: The gateway SHALL register GTS schemas for platform and cache plugins during initialization.

**Implementation details**:
- Platform schema ID: `gts.x.core.modkit.plugin.v1~x.core.license_enforcer.integration.plugin.v1~`
- Cache schema ID: `gts.x.core.modkit.plugin.v1~x.core.license_enforcer.cache.plugin.v1~`

**Implements**:
- `fdd-license-enforcer-feature-plugin-discovery-flow-gateway-register-schemas`

**Phases**:
- [ ] `ph-1`: Register schemas and gateway client
<!-- fdd-id-content -->

### Support Scoped ClientHub Registration

- [ ] **ID**: `fdd-license-enforcer-feature-plugin-discovery-req-scoped-clienthub`

<!-- fdd-id-content -->
**Status**: COMPLETED

**Description**: Platform and cache plugins SHALL register their client implementations in `ClientHub` scoped by their GTS instance ID.

**Implementation details**:
- Platform client trait: `PlatformPluginClient`
- Cache client trait: `CachePluginClient`
- Scope: `ClientScope::gts_id(instance_id)`

**Implements**:
- `fdd-license-enforcer-feature-plugin-discovery-flow-plugin-registers`

**Phases**:
- [ ] `ph-1`: Scoped client registration
<!-- fdd-id-content -->

### Resolve Plugins via Types Registry

- [ ] **ID**: `fdd-license-enforcer-feature-plugin-discovery-req-resolve-plugins`

<!-- fdd-id-content -->
**Status**: COMPLETED

**Description**: The gateway SHALL discover plugin instances via `types-registry` and resolve their scoped clients from `ClientHub` using the selected instance ID.

**Implementation details**:
- Uses vendor + priority selection over `BaseModkitPluginV1<Spec>` entries

**Implements**:
- `fdd-license-enforcer-feature-plugin-discovery-algo-choose-plugin-instance`

**Phases**:
- [ ] `ph-1`: Lazy resolution and caching of plugin instance IDs
<!-- fdd-id-content -->
