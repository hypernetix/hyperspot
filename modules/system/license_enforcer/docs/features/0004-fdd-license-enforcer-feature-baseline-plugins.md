# Feature: Baseline Plugins and Configuration

## 1. Feature Context

**ID**: `fdd-license-enforcer-feature-baseline-plugins`

**Status**: NON IMPLEMENTED

### 1.1 Overview

Provide baseline plugin implementations that make license enforcement usable out-of-the-box:
- Platform integration: static licenses from configuration
- Cache: no-cache (always miss)
- Cache: in-memory (TTL + max entries)

### 1.2 Purpose

Ensure the gateway can operate in local/dev environments and minimal deployments without requiring a custom Platform integration plugin.

### 1.3 Actors

- `fdd-license-enforcer-actor-system-operator` - Configures plugins via module configuration
- `fdd-license-enforcer-actor-plugin` - Implements baseline plugins

### 1.4 References

- Overall Design: [DESIGN.md](../DESIGN.md)
- Dependencies: `fdd-license-enforcer-feature-plugin-discovery`, `fdd-license-enforcer-feature-cache-aside`

## 2. Actor Flows (FDL)

### Configure Static Licenses Plugin

- [ ] `ph-1` **ID**: `fdd-license-enforcer-feature-baseline-plugins-flow-configure-static-licenses`

<!-- fdd-id-content -->
**Actor**: `fdd-license-enforcer-actor-system-operator`

**Success Scenarios**:
- Operator configures a list of enabled global features; plugin returns them plus the base feature.

**Error Scenarios**:
- Invalid GTS IDs in configuration cause plugin init failure.

**Steps**:
1. [ ] - `ph-1` - Operator configures `static_licenses_features` list for `static_licenses_plugin`. - `inst-baseline-plugins-static-1`
2. [ ] - `ph-1` - Plugin loads configuration via `ctx.config()?`. - `inst-baseline-plugins-static-2`
3. [ ] - `ph-1` - Plugin validates each entry as a structurally valid GTS ID. - `inst-baseline-plugins-static-3`
4. [ ] - `ph-1` - Plugin registers its instance in `types-registry` and scoped client in `ClientHub`. - `inst-baseline-plugins-static-4`
5. [ ] - `ph-1` - **RETURN** enabled features = base + configured features. - `inst-baseline-plugins-static-5`
<!-- fdd-id-content -->

### Configure In-Memory Cache Plugin TTL

- [ ] `ph-1` **ID**: `fdd-license-enforcer-feature-baseline-plugins-flow-configure-inmemory-ttl`

<!-- fdd-id-content -->
**Actor**: `fdd-license-enforcer-actor-system-operator`

**Success Scenarios**:
- Cached tenant feature sets expire after configured TTL.

**Error Scenarios**:
- Unknown config fields are rejected (deny unknown fields).

**Steps**:
1. [ ] - `ph-1` - Operator configures `ttl` and `max_entries` for `inmemory_cache_plugin`. - `inst-baseline-plugins-cache-1`
2. [ ] - `ph-1` - Plugin loads configuration via `ctx.config()?`. - `inst-baseline-plugins-cache-2`
3. [ ] - `ph-1` - Plugin initializes service with configured TTL and entry limit. - `inst-baseline-plugins-cache-3`
4. [ ] - `ph-1` - Plugin registers instance + scoped client. - `inst-baseline-plugins-cache-4`
<!-- fdd-id-content -->

## 3. Algorithms (FDL)

### Static Licenses Feature List Normalization

- [ ] **ID**: `fdd-license-enforcer-feature-baseline-plugins-algo-static-features`

<!-- fdd-id-content -->
**Input**: `static_licenses_features` list from configuration.

**Output**: List of `LicenseFeatureID` returned by the plugin (plus base feature).

**Steps**:
1. [ ] - `ph-1` - Validate each configured string is a valid GTS ID. - `inst-baseline-plugins-algo-1`
2. [ ] - `ph-1` - Convert each string into `LicenseFeatureID`. - `inst-baseline-plugins-algo-2`
3. [ ] - `ph-1` - Add base feature ID to returned set. - `inst-baseline-plugins-algo-3`
4. [ ] - `ph-1` - **RETURN** normalized list. - `inst-baseline-plugins-algo-4`
<!-- fdd-id-content -->

## 4. States (FDL)

No state machine. Plugin state is limited to in-process service data structures (e.g., cache map).

## 5. Requirements

### Provide Baseline Plugin Crates

- [ ] **ID**: `fdd-license-enforcer-feature-baseline-plugins-req-baseline-crates`

<!-- fdd-id-content -->
**Status**: COMPLETED

**Description**: The system SHALL provide baseline plugin crates:
- Platform integration: `plugins/builtin-integration/static_licenses_plugin`
- Cache: `plugins/cache/nocache_plugin`
- Cache: `plugins/cache/inmemory_cache_plugin`

**Implementation details**:
- Each plugin registers an instance in `types-registry` and a scoped client in `ClientHub`.

**Phases**:
- [ ] `ph-1`: Baseline plugins are buildable and usable
<!-- fdd-id-content -->

### Baseline Plugin Instance IDs

- [ ] **ID**: `fdd-license-enforcer-feature-baseline-plugins-req-instance-ids`

<!-- fdd-id-content -->
**Status**: COMPLETED

**Description**: The baseline plugins SHALL register the following instance IDs:
- `gts.x.core.modkit.plugin.v1~x.core.license_enforcer.integration.plugin.v1~hyperspot.builtin.static_licenses.integration.plugin.v1`
- `gts.x.core.modkit.plugin.v1~x.core.license_enforcer.cache.plugin.v1~hyperspot.builtin.nocache.cache.plugin.v1`
- `gts.x.core.modkit.plugin.v1~x.core.license_enforcer.cache.plugin.v1~hyperspot.builtin.inmemory.cache.plugin.v1`

**Phases**:
- [ ] `ph-1`: Fixed instance IDs for builtin plugins
<!-- fdd-id-content -->

### Static Licenses Configurable Enabled Features

- [ ] **ID**: `fdd-license-enforcer-feature-baseline-plugins-req-static-config`

<!-- fdd-id-content -->
**Status**: COMPLETED

**Description**: The baseline `static_licenses_plugin` SHALL be configurable with an explicit list of enabled global features (`static_licenses_features`), in addition to the always-enabled base feature.

If `static_licenses_features` is omitted, the plugin SHALL behave as if an empty list was configured (base feature only).

**Implementation details**:
- Configuration loaded via `ctx.config()?` during module initialization.
- GTS ID structure validation is performed during module initialization.

**Implements**:
- `fdd-license-enforcer-feature-baseline-plugins-flow-configure-static-licenses`
- `fdd-license-enforcer-feature-baseline-plugins-algo-static-features`

**Phases**:
- [ ] `ph-1`: Static feature list configuration
<!-- fdd-id-content -->

## 6. Additional Context (optional)

### Known Spec/Implementation Divergence

**ID**: `fdd-license-enforcer-feature-baseline-plugins-context-spec-impl-divergence`

<!-- fdd-id-content -->
The OpenSpec requirement allows `static_licenses_features` to be omitted (treated as an empty list). The current `static_licenses_plugin` configuration parser requires the field to be present. If you want strict spec alignment at runtime, the plugin config parsing needs to accept the field as optional and default it to `[]`.
<!-- fdd-id-content -->
