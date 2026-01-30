# Static Licenses Plugin

Static licenses plugin for the license enforcer gateway. This plugin provides configurable license feature data for bootstrap, testing, and local development purposes.

## Overview

The `static_licenses_plugin` is a platform integration plugin that returns a configured set of enabled global features. Unlike a real platform integration that would query an external licensing service, this plugin returns a static, configuration-driven feature set.

## Use Cases

- **Bootstrap**: Minimal license configuration for system startup
- **Local Development**: Simulate different license scenarios without external dependencies
- **Testing**: Controlled, reproducible license states for integration tests

## Configuration

The plugin requires the following configuration:

```yaml
static_licenses_plugin:
  # Vendor identifier (optional, defaults to "hyperspot")
  vendor: "hyperspot"

  # Plugin selection priority (optional, defaults to 100)
  # Lower values have higher priority
  priority: 100

  # List of enabled global features (REQUIRED)
  # HyperSpot GTS identifiers for licensed features
  # An empty list is valid (returns base feature only)
  static_licenses_features:
    - "gts.hyperspot.feature.advanced-analytics.v1"
    - "gts.hyperspot.feature.export.v1"
```

### Configuration Fields

#### `vendor` (optional)
- **Type**: String
- **Default**: `"hyperspot"`
- **Description**: Vendor identifier for the plugin instance

#### `priority` (optional)
- **Type**: Integer
- **Default**: `100`
- **Description**: Plugin selection priority. Lower values have higher priority when multiple plugins are available.

#### `static_licenses_features` (required)
- **Type**: Array of strings (GTS IDs)
- **Description**: List of HyperSpot GTS IDs for enabled global features
- **Important**: This field is REQUIRED. If omitted, module initialization will fail with a configuration error.
- **Note**: An empty array is valid and will result in only the base feature being enabled.
- **Validation**: During module initialization, GTS ID format validation is performed using the `gts` crate:
  - Each ID must be a valid GTS identifier (proper structure, segments, version, etc.)
  - Validation uses `gts::GtsID::is_valid()` for spec-compliant checking
  - No registry validation is performed (structure validation only)

## Behavior

### Feature Resolution

The plugin returns enabled global features as follows:
1. Always includes the **base feature** (`gts.hyperspot.license_enforcer.base_feature.v1`)
2. Includes all features listed in `static_licenses_features`
3. Deduplicates features if the base feature is included in the configuration

### Error Handling

- **Missing tenant scope**: Returns `MissingTenantScope` error if called without a valid tenant context
- **Missing configuration**: Fails module initialization if `static_licenses_features` field is not provided

## Examples

### Minimal Configuration (Base Feature Only)

```yaml
static_licenses_plugin:
  static_licenses_features: []
```

This configuration enables only the base feature.

### Development Configuration (Multiple Features)

```yaml
static_licenses_plugin:
  vendor: "hyperspot"
  priority: 50
  static_licenses_features:
    - "gts.hyperspot.feature.advanced-analytics.v1"
    - "gts.hyperspot.feature.export.v1"
    - "gts.hyperspot.feature.api-access.v1"
```

This configuration enables the base feature plus three additional features with a custom priority.

### Testing Configuration (Custom Vendor)

```yaml
static_licenses_plugin:
  vendor: "test-vendor"
  priority: 200
  static_licenses_features:
    - "gts.custom.feature.test.v1"
```

This configuration is useful for testing plugin selection logic with a custom vendor.

## Module Registration

The plugin automatically registers itself in the types registry with:
- **Plugin Type**: `gts.hyperspot.license_enforcer.platform_integration.plugin.v1`
- **Instance ID**: Generated based on the plugin type
- **Client Registration**: Scoped client in ClientHub for gateway integration

## Dependencies

- `license-enforcer-sdk`: Core SDK for license enforcement
- `types-registry-sdk`: For plugin registration
- `modkit`: Module framework
- `modkit-security`: Security context handling

## See Also

- [License Enforcer Gateway](../../../license_enforcer-gw/README.md)
- [License Enforcer SDK](../../../license_enforcer-sdk/README.md)
